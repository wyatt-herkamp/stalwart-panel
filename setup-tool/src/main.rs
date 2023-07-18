use std::collections::HashMap;
use clap::builder::Str;
use clap::{Parser, ValueEnum};
use entities::emails::EmailType;
use entities::groups::GroupPermissions;
use entities::{AccountEntity, ActiveAccountModel, ActiveEmailModel, EmailEntity, GroupEntity};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveValue, ConnectOptions, DatabaseConnection, EntityTrait};
use sqlx::{Any, AnyConnection, Connection, Sqlite, SqliteConnection};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;
use log::{debug, info};
use toml_edit::{Document, Item};
use entities::account::AccountType;
use utils::config::{Database, MysqlSettings, PostgresSettings};

#[derive(ValueEnum, Debug, Clone)]
pub enum DatabaseType {
    Mysql,
    Postgres,
}
#[derive(Parser)]
struct Command {
    #[clap(long)]
    stalwart_config: PathBuf,
    #[clap(long)]
    database_type: DatabaseType,
    #[clap(long)]
    database_host: String,
    #[clap(long)]
    database_user: String,
    #[clap(long)]
    database_password: String,
    #[clap(long)]
    database_name: String,
}
#[tokio::main]
async fn main() {
    let command = Command::parse();
    let toml_content =
        read_to_string(command.stalwart_config).expect("Failed to read stalwart config file");

    let database_config = match command.database_type {
        DatabaseType::Mysql => Database::Mysql(MysqlSettings {
            user: command.database_user,
            password: command.database_password,
            host: command.database_host,
            database: command.database_name,
        }),
        DatabaseType::Postgres => Database::Postgres(PostgresSettings {
            user: command.database_user,
            password: command.database_password,
            host: command.database_host,
            database: command.database_name,
        }),
    };

    let mut database_connection =
        sea_orm::Database::connect(ConnectOptions::new(database_config.to_string()))
            .await
            .expect("Failed to connect to database");

    Migrator::up(&database_connection, None)
        .await
        .expect("Failed to run migrations");
    let mut stalwart_config: Document = toml_content
        .parse()
        .expect("Failed to parse stalwart config file");

    {
        let group = entities::ActiveGroupModel {
            id: ActiveValue::NotSet,
            group_name: ActiveValue::Set("user".to_string()),
            permissions: ActiveValue::Set(GroupPermissions::default()),
            created: ActiveValue::Set(Default::default()),
        };

        GroupEntity::insert(group)
            .exec(&database_connection)
            .await
            .expect("Failed to insert user group");
    }
    let super_user_name = stalwart_config
        .get("directory.sql.options.superuser-group")
        .and_then(|item| item.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "superuser".to_string());
    {

        let group = entities::ActiveGroupModel {
            id: ActiveValue::NotSet,
            group_name: ActiveValue::Set(super_user_name.clone()),
            permissions: ActiveValue::Set(GroupPermissions::new_admin()),
            created: ActiveValue::Set(Default::default()),
        };

        GroupEntity::insert(group)
            .exec(&database_connection)
            .await
            .expect("Failed to insert superuser group");
    }

    let mut old_database = stalwart_config.get("directory.sql.address");

    match old_database {
        None => {
            println!("No old database found, No database to import");

            create_default_account(&database_connection).await;
        }
        Some(database) => {
            let database = database.as_str().expect("Failed to parse database address");

            import_database(&mut database_connection, database, &super_user_name).await;
        }
    }

    // TODO add the new database to the config file
}

async fn create_default_account(database_connection: &DatabaseConnection) {
    let postmaster = ActiveAccountModel {
        id: Default::default(),
        username: ActiveValue::Set("postmaster".to_string()),
        description: ActiveValue::Set("Postmaster Account".to_string()),
        group_id: ActiveValue::Set(2),
        password: ActiveValue::Set("".to_string()),
        quota: ActiveValue::Set(0),
        account_type: ActiveValue::Set(AccountType::Individual),
        active: ActiveValue::Set(true),
        backup_email: Default::default(),
        created: Default::default(),
    };

    let id = entities::AccountEntity::insert(postmaster)
        .exec(&database_connection)
        .await
        .expect("Failed to insert postmaster account")
        .last_insert_id;

    let postmaster_email = entities::ActiveEmailModel {
        id: Default::default(),
        email: ActiveValue::Set("postmaster@localhost".to_string()),
        created: Default::default(),
        account: ActiveValue::Set(id),
        email_type: ActiveValue::Set(EmailType::Primary),
    };

    entities::EmailEntity::insert(postmaster_email)
        .exec(&database_connection)
        .await
        .expect("Failed to insert postmaster email");
}

/// Table Layout for the Account Table
///
/// - Name
/// - Secret
/// - Description
/// - Type
/// - Quota
/// - Active
 type OldAccount = (String, String, String, String,i64, bool);
/// Table Layout for the Email Table
/// - Name
/// - Address
/// - Type
 type OldEmail = (String, String, String);

/// Imports the Default Stalwart Database to the new database
///
/// ## Notes
///  - We do not support a user being in multiple groups. And it defaults everyone to the user group unless they are a superuser
///  - This method fails however the old database is not modified. So if something happens just drop the new database and re run it
///  - Email Types Supported are Primary and Alias
///  - Account Types Supported are Individual(Will convert Personal to Individual) and Group
async fn import_database(database: &mut DatabaseConnection, old_database: &str, superuser: &str) {
    info!("Loading Old Database");
    let mut old_database = AnyConnection::connect(old_database)
        .await
        .expect("Failed to connect to old database");
    info!("Loading Old Database Tables");
    let old_accounts: Vec<OldAccount> =
        sqlx::query_as("SELECT * FROM accounts")
            .fetch_all(&mut old_database)
            .await
            .expect("Failed to fetch accounts");
    debug!("Found {} Accounts", old_accounts.len());
    let group_members: Vec<(String, String)> = sqlx::query_as("SELECT * FROM group_members")
        .fetch_all(&mut old_database)
        .await
        .expect("Failed to fetch group members");
    debug!("Found {} Group Members", group_members.len());
    let old_emails: Vec<OldEmail> = sqlx::query_as("SELECT * FROM emails")
        .fetch_all(&mut old_database)
        .await
        .expect("Failed to fetch emails");
    debug!("Found {} Emails", old_emails.len());
    // Stalwart allows users to be apart of multiple groups, but we don't support that yet
    // By default this is going to make anyone with a superuser group a superuser. Then everyone else default
    // Stalwart is not production ready so I doubt anyone has some massive user base with multiple groups
    let group_members: HashMap<String, i64> = group_members.into_iter().map(|(name, group)|{
        if group == superuser{
            (name, 2)
        } else {
            (name, 1)
        }
    }).collect();

    let new_accounts = old_accounts
        .into_iter()
        .map(|(username, password, description, t,quota,  active)| {
            let account_type = AccountType::from_str(&t).expect("Failed to parse account type");
            let group_id = group_members.get(&username).copied().unwrap_or(1);
            // Yes I am cloning. I am really lazy
            (username.clone(), ActiveAccountModel {
                id: Default::default(),
                username: ActiveValue::Set(username),
                description: ActiveValue::Set(description),
                group_id: ActiveValue::Set(group_id),
                password: ActiveValue::Set(password),
                quota: ActiveValue::Set(quota),
                account_type: ActiveValue::Set(account_type),
                active: ActiveValue::Set(active),
                backup_email: ActiveValue::Set(None),
                created: Default::default(),
            })
        })
        .collect::<Vec<(String, ActiveAccountModel)>>();


    for (name, account) in new_accounts{
        // TODO once drain filter is stabilized move to it https://github.com/rust-lang/rust/issues/43244
        let collected_emails = old_emails.iter().filter(|e| {
            e.0 == name
        }).map(|e| {
            let email_type = EmailType::from_str(&e.2).unwrap_or(EmailType::Alias);
            (e.1.clone(), email_type)
        }).collect::<Vec<(String, EmailType)>>();

        let id = AccountEntity::insert(account)
            .exec(database)
            .await
            .expect("Failed to insert account").last_insert_id;
        info!("Inserted Account {}", name);
        // Add all of the emails
        for (email, email_type) in collected_emails{
            debug!("Inserting Email {} for Account {}", email, name);
            let email = ActiveEmailModel{
                id: Default::default(),
                account: ActiveValue::Set(id),
                email: ActiveValue::Set(email),
                email_type: ActiveValue::Set(email_type),
                created: Default::default(),
            };

            EmailEntity::insert(email)
                .exec(database)
                .await
                .expect("Failed to insert email");
        }
    }
}
