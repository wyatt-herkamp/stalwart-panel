use clap::{Parser, ValueEnum};
use entities::account::AccountType;
use entities::emails::EmailType;
use entities::groups::{ActiveModel, GroupPermissions};
use entities::{
    now, AccountEntity, ActiveAccountModel, EmailActiveModel, EmailEntity, GroupEntity,
};
use log::{debug, error, info};
use migration::{Migrator, MigratorTrait};
use rand::distributions::Distribution;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, ConnectOptions, DatabaseConnection, EntityTrait};


use sqlx::{Connection, SqliteConnection};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;
use toml_edit::{Document, Item};
use utils::config::{
    Database, EmailEncryption, EmailSetting, MysqlSettings, PostgresSettings, Settings,
    StalwartManagerConfig,
};
use utils::database::{EmailAddress, Password};
use utils::stalwart_config::sql::{SQLColumns, SQLQuery};
use utils::stalwart_manager::ManagerConfig;

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
    #[clap(long, default_value = "true")]
    import: bool,
}

#[tokio::main]
async fn main() {
    let command = Command::parse();
    sqlx::any::install_default_drivers();
    let toml_content =
        read_to_string(&command.stalwart_config).expect("Failed to read stalwart config file");

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

    let stalwart_config: Document = toml_content
        .parse()
        .expect("Failed to parse stalwart config file");

    let super_user_name = stalwart_config["directory"]["sql"]["options"]["superuser-group"]
        .as_str()
        .unwrap_or("superuser");
    setup_database(&mut database_connection, super_user_name).await;

    let password = if command.import {
        let old_database = stalwart_config["directory"]["sql"]["address"]
            .as_str()
            .expect("Failed to parse old database address");

        info!("Importing Old Database");
        import_database(&mut database_connection, old_database, &super_user_name).await;
        "<Please Provide Password>".to_string()
    } else {
        info!("Skipping Import. Creating Default Account");
        create_default_account(&mut database_connection).await
    };
    // TODO add the new database to the config file

    tokio::spawn(async move {
        if let Err(e) = database_connection.close().await {
            error!("Failed to close database connection {}", e);
        }
    });

    let main_domain =
        if let Some(array) = stalwart_config["directory"]["imap"]["lookup"]["domains"].as_array() {
            array
                .get(0)
                .and_then(|item| item.as_str())
                .unwrap_or("example.com")
        } else {
            "example.com"
        };
    let postmaster_address = format!("postmaster@{}", main_domain);

    let email = EmailSetting {
        username: "postmaster".to_string(),
        password,
        host: format!(
            "{}:587",
            stalwart_config["server"]["hostname"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or("localhost".to_string())
        ),
        encryption: EmailEncryption::StartTLS,
        from: format!("Stalwart Panel<panel@{}>", main_domain),
        reply_to: None,
    };
    update_config(&database_config, stalwart_config, &command.stalwart_config);

    info!("Stalwart has been configured. Creating the panel config");

    let config = Settings::new(database_config, postmaster_address, email);

    let config = toml::to_string_pretty(&config).expect("Failed to serialize config");

    let config_file = PathBuf::from("stalwart-panel.toml");
    if config_file.exists() {
        std::fs::remove_file(&config_file).expect("Failed to remove old config file");
    }
    std::fs::write(&config_file, &config).expect("Failed to write config file");

    let stalwart_manager_config = ManagerConfig {
        stalwart_config: command.stalwart_config,
        ..StalwartManagerConfig::default()
    };

    let stalwart_manager_config =
        toml::to_string_pretty(&stalwart_manager_config).expect("Failed to serialize config");

    let stalwart_manager_config_file = PathBuf::from("stalwart-manager.toml");

    if stalwart_manager_config_file.exists() {
        std::fs::remove_file(&stalwart_manager_config_file)
            .expect("Failed to remove old config file");
    }

    std::fs::write(&stalwart_manager_config_file, &stalwart_manager_config)
        .expect("Failed to write config file");

    info!("Stalwart Panel has been configured. Please double check stalwart-panel.toml and then run stalwart-panel");
}

async fn setup_database(database_connection: &DatabaseConnection, super_user_name: &str) {
    // Drop old tables

    Migrator::up(database_connection, None)
        .await
        .expect("Failed to run migrations");

    {
        let group = entities::ActiveGroupModel {
            id: ActiveValue::Set(1),
            group_name: ActiveValue::Set("user".to_string()),
            permissions: ActiveValue::Set(GroupPermissions::default()),
            created: ActiveValue::Set(Default::default()),
        };
        debug!("Inserting User Group {:?}", group);
        insert_group(database_connection, group).await;
    }
    {
        let group = entities::ActiveGroupModel {
            id: ActiveValue::Set(2),
            group_name: ActiveValue::Set(super_user_name.to_string()),
            permissions: ActiveValue::Set(GroupPermissions::new_admin()),
            created: ActiveValue::Set(Default::default()),
        };
        debug!("Inserting User Group {:?}", group);
        insert_group(database_connection, group).await;
    }
}

async fn insert_group(database_connection: &DatabaseConnection, group: ActiveModel) {
    use entities::groups::Column;
    GroupEntity::insert(group)
        .on_conflict(
            OnConflict::column(Column::Id)
                .update_columns(vec![Column::GroupName, Column::Permissions])
                .to_owned(),
        )
        .exec(database_connection)
        .await
        .expect("Failed to insert user group");
}

/// Tests to see if the config file is updated correctly
#[test]
pub fn test_config_update() {
    let current_dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .parent()
        .unwrap()
        .to_path_buf();
    let file = current_dir.join("test_data").join("stalwart-config.toml");
    assert!(file.exists(), "Test file does not exist {}", file.display());
    let copy = current_dir
        .join("test_data")
        .join("stalwart-config-test.toml");
    if copy.exists() {
        std::fs::remove_file(&copy).expect("Failed to remove test file");
    }
    std::fs::copy(&file, &copy).expect("Failed to copy file");

    drop(file);
    let toml_content = read_to_string(&copy).expect("Failed to read stalwart config file");

    let document: Document = toml_content
        .parse()
        .expect("Failed to parse stalwart config file");

    let database = Database::test();

    update_config(&database, document, &copy);
}

fn update_config(database_config: &Database, mut stalwart_config: Document, file: &PathBuf) {
    stalwart_config["directory"]["sql"]["address"] =
        Item::Value(database_config.to_string().into());

    let queries = match &database_config {
        Database::Mysql(_) => SQLQuery::new_mysql(),
        Database::Postgres(_) => SQLQuery::new_postgres(),
    };
    stalwart_config["directory"]["sql"]["queries"] = queries.into();
    stalwart_config["directory"]["sql"]["columns"] = SQLColumns::default().into();

    // Backup the old config file
    let backup_file = file.with_extension("bak");
    if backup_file.exists() {
        std::fs::remove_file(&backup_file).expect("Failed to remove old backup file");
    }
    std::fs::rename(&file, &backup_file).expect("Failed to backup stalwart config file");
    let content = stalwart_config.to_string();
    std::fs::write(file, &content).expect("Failed to write new stalwart config file");
}

async fn create_default_account(database_connection: &mut DatabaseConnection) -> String {
    // Generate Random Password

    let mut rng = StdRng::from_entropy();
    let password: String = rand::distributions::Alphanumeric
        .sample_iter(&mut rng)
        .take(16)
        .map(char::from)
        .collect();

    // Argon2 Hash the password

    let password_hashed = Password::new_argon2(&password).expect("Failed to hash password");

    let postmaster = ActiveAccountModel {
        id: Default::default(),
        name: ActiveValue::Set("Postmaster".to_string()),
        username: ActiveValue::Set("postmaster".to_string()),
        description: ActiveValue::Set("Postmaster Account".to_string()),
        group_id: ActiveValue::Set(2),
        password: ActiveValue::Set(password_hashed),
        require_password_change: ActiveValue::Set(false),
        quota: ActiveValue::Set(0),
        account_type: ActiveValue::Set(AccountType::Individual),
        active: ActiveValue::Set(true),
        backup_email: Default::default(),
        created: now(),
    };
    AccountEntity::insert(postmaster)
        .exec(database_connection)
        .await
        .expect("Failed to insert postmaster account");

    password
}

/// Table Layout for the Account Table
///
/// - Name
/// - Secret
/// - Description
/// - Type
/// - Quota
/// - Active
type OldAccount = (String, String, String, String, i64, bool);
/// Table Layout for the Email Table
/// - Name
/// - Address
/// - Type
type OldEmail = (String, String, String);
async fn get_old_data_sqlite(
    old_database: &str,
) -> (Vec<OldAccount>, Vec<(String, String)>, Vec<OldEmail>) {
    let mut old_database = SqliteConnection::connect(old_database)
        .await
        .expect("Failed to connect to old database");

    let old_accounts: Vec<OldAccount> = sqlx::query_as("SELECT * FROM accounts")
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

    (old_accounts, group_members, old_emails)
}
/// Imports the Default Stalwart Database to the new database
///
/// ## Notes
///  - We do not support a user being in multiple groups. And it defaults everyone to the user group unless they are a superuser
///  - This method fails however the old database is not modified. So if something happens just drop the new database and re run it
///  - Email Types Supported are Primary and Alias
///  - Account Types Supported are Individual(Will convert Personal to Individual) and Group
async fn import_database(database: &mut DatabaseConnection, old_database: &str, superuser: &str) {
    info!("Loading Old Database");

    let (old_accounts, group_members, old_emails) = if old_database.contains("sqlite") {
        get_old_data_sqlite(old_database).await
    } else {
        todo!("Add support for other databases")
    };

    // Stalwart allows users to be apart of multiple groups, but we don't support that yet
    // By default this is going to make anyone with a superuser group a superuser. Then everyone else default
    // Stalwart is not production ready so I doubt anyone has some massive user base with multiple groups
    let group_members: HashMap<String, i64> = group_members
        .into_iter()
        .map(|(name, group)| {
            if group == superuser {
                (name, 2)
            } else {
                (name, 1)
            }
        })
        .collect();

    let new_accounts = old_accounts
        .into_iter()
        .map(|(username, password, description, t, quota, active)| {
            let account_type = AccountType::from_str(&t).expect("Failed to parse account type");
            let group_id = group_members.get(&username).copied().unwrap_or(1);
            // Yes I am cloning. I am really lazy
            (
                username.clone(),
                ActiveAccountModel {
                    id: Default::default(),
                    name: ActiveValue::Set(username.clone()),
                    username: ActiveValue::Set(username),
                    description: ActiveValue::Set(description),
                    group_id: ActiveValue::Set(group_id),
                    password: ActiveValue::Set(Password::new_hashed(password)),
                    require_password_change: ActiveValue::Set(false),
                    quota: ActiveValue::Set(quota),
                    account_type: ActiveValue::Set(account_type),
                    active: ActiveValue::Set(active),
                    backup_email: ActiveValue::Set(None),
                    created: now(),
                },
            )
        })
        .collect::<Vec<(String, ActiveAccountModel)>>();

    for (name, account) in new_accounts {
        // TODO once drain filter is stabilized move to it https://github.com/rust-lang/rust/issues/43244
        let collected_emails = old_emails
            .iter()
            .filter(|e| e.0 == name)
            .map(|e| {
                let email_type = EmailType::from_str(&e.2).unwrap_or(EmailType::Alias);
                (e.1.clone(), email_type)
            })
            .collect::<Vec<(String, EmailType)>>();

        let id = AccountEntity::insert(account)
            .on_conflict(
                OnConflict::column(entities::account::Column::Username)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(database)
            .await
            .expect("Failed to insert account")
            .last_insert_id;
        info!("Inserted Account {}", name);
        // Add all of the emails
        for (email, email_type) in collected_emails {
            debug!("Inserting Email {} for Account {}", email, name);

            let email = EmailActiveModel {
                id: Default::default(),
                account: ActiveValue::Set(id),
                email_address: ActiveValue::Set(EmailAddress::new(email).unwrap()),
                email_type: ActiveValue::Set(email_type),
                created: now(),
            };

            EmailEntity::insert(email)
                .exec(database)
                .await
                .expect("Failed to insert email");
        }
    }
}
