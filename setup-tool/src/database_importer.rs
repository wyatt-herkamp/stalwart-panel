use std::collections::HashMap;
use std::str::FromStr;

use inquire::Confirm;
use log::{debug, error, info, warn};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use sqlx::{postgres, Connection, FromRow, SqliteConnection};

use crate::Error;
use entities::account::AccountType;
use entities::emails::EmailType;
use entities::{now, AccountEntity, ActiveAccountModel, EmailActiveModel, EmailEntity};
use utils::database::password::PasswordType;
use utils::database::{EmailAddress, Password};

/// Table Layout for the Account Table
///
/// - Name
/// - Secret
/// - Description
/// - Type
/// - Quota
/// - Active
#[derive(Debug, FromRow)]
pub struct OldAccount {
    pub name: String,
    pub secret: String,
    pub description: String,
    #[sqlx(rename = "type")]
    pub account_type: String,
    pub quota: i64,
    pub active: bool,
}
/// Table Layout for the Email Table
/// - Name
/// - Address
/// - Type
#[derive(Debug, FromRow)]
pub struct OldEmail {
    pub name: String,
    pub address: String,
    #[sqlx(rename = "type")]
    pub account_type: String,
}
#[derive(Debug)]
pub struct Import {
    pub accounts: Vec<OldAccount>,
    pub group_members: Vec<(String, String)>,
    pub emails: Vec<OldEmail>,
}
async fn get_old_data_sqlite(old_database: &str) -> Result<Import, Error> {
    let mut old_database = SqliteConnection::connect(old_database)
        .await
        .expect("Failed to connect to old database");

    let old_accounts: Vec<OldAccount> = sqlx::query_as("SELECT * FROM accounts")
        .fetch_all(&mut old_database)
        .await?;

    debug!("Found {} Accounts", old_accounts.len());
    let group_members: Vec<(String, String)> = sqlx::query_as("SELECT * FROM group_members")
        .fetch_all(&mut old_database)
        .await?;
    debug!("Found {} Group Members", group_members.len());
    let old_emails: Vec<OldEmail> = sqlx::query_as("SELECT * FROM emails")
        .fetch_all(&mut old_database)
        .await?;
    debug!("Found {} Emails", old_emails.len());

    Ok(Import {
        accounts: old_accounts,
        group_members,
        emails: old_emails,
    })
}
async fn get_old_data_postgres(old_database: &str) -> Result<Import, Error> {
    let mut old_database = postgres::PgConnection::connect(old_database).await?;

    let old_accounts: Vec<OldAccount> = sqlx::query_as("SELECT * FROM accounts")
        .fetch_all(&mut old_database)
        .await?;

    debug!("Found {} Accounts", old_accounts.len());
    let group_members: Vec<(String, String)> = sqlx::query_as("SELECT * FROM group_members")
        .fetch_all(&mut old_database)
        .await?;
    debug!("Found {} Group Members", group_members.len());
    let old_emails: Vec<OldEmail> = sqlx::query_as("SELECT * FROM emails")
        .fetch_all(&mut old_database)
        .await?;
    debug!("Found {} Emails", old_emails.len());

    Ok(Import {
        accounts: old_accounts,
        group_members,
        emails: old_emails,
    })
}
/// Imports the Default Stalwart Database to the new database
///

/// ## Returns
/// - Ok(true) - If there was an error importing the database
/// - Ok(false) - If there was no error importing the database
/// - Err(Err) - If unable to pull data from the old database
///
/// ## Notes
///  - We do not support a user being in multiple groups.
/// And it defaults everyone to the user group unless they are a superuser
///  - This method fails however the old database is not modified.
/// So if something happens just drop the new database and re run it
///  - Email Types Supported are Primary and Alias
///  - Account Types Supported are Individual(Will convert Personal to Individual) and Group
pub(crate) async fn import_database(
    database: &mut DatabaseConnection,
    old_database: &str,
    superuser: &str,
    no_questions_asked: bool,
    require_password_changes_on_all_users: bool,
) -> Result<bool, Error> {
    info!("Loading Old Database");

    let Import {
        accounts,
        group_members,
        emails,
    } = if old_database.contains("sqlite") {
        get_old_data_sqlite(old_database).await
    } else if old_database.contains("postgres") {
        get_old_data_postgres(old_database).await
    } else {
        todo!("Add support for other databases")
    }?;
    let mut has_errors = false;

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

    let new_accounts = accounts
        .into_iter()
        .map(|OldAccount{ name, secret, description, account_type, quota, active }| {
            let account_type = AccountType::from_str(&account_type).expect("Failed to parse account type");
            let group_id = group_members.get(&name).copied().unwrap_or(1);
            // Yes I am cloning. I am really lazy
            let mut hashed_password = Password::new_hashed(&secret);
            match hashed_password.hash_type() {
                PasswordType::PlainText => {
                    if !no_questions_asked{
                        if let Ok(value) = Confirm::new(
                            r#"Password is in plain text. Would you like to hash it using Argon2. 
                            If not you will need to reset your password to access the Stalwart Panel"#
                        )
                            .with_default(false)
                            .prompt()
                        {
                            if value {
                                let result = Password::new_hash(secret, PasswordType::Argon2);
                                match result {
                                    Ok(ok) => hashed_password = ok,
                                    Err(err) => {
                                        error!(
                                            r#"Failed to hash password for user {name}
                                            The users password will stay as plain text.
                                            The user will need to reset their password to access the Stalwart Panel.
                                            Error: {err}"#);
                                        // This Error is not fatal. So we will continue
                                    }
                                }
                            }
                        };
                    }else{
                        warn!(
                            r#"Account {name} has a plain text password.
                             This user will not able to log into Stalwart Panel Until it is updated"#
                        )
                    }

                }
                PasswordType::None => {
                    warn!(
                        r#"Account {name} has an unsupported Password Hash for Stalwart Panel.
                Accessing your Email account will continue to work,
                However, you will need to reset your password to access the Stalwart Panel"#
                    );
                }
                _ => {}
            }
            (
                name.clone(),
                ActiveAccountModel {
                    id: Default::default(),
                    name: ActiveValue::Set(name.clone()),
                    username: ActiveValue::Set(name),
                    description: ActiveValue::Set(description),
                    group_id: ActiveValue::Set(group_id),
                    password: ActiveValue::Set(hashed_password),
                    require_password_change: ActiveValue::Set(require_password_changes_on_all_users),
                    quota: ActiveValue::Set(quota),
                    account_type: ActiveValue::Set(account_type),
                    active: ActiveValue::Set(active),
                    backup_email: Default::default(),
                    created: Default::default(),
                },
            )
        })
        .collect::<Vec<(String, ActiveAccountModel)>>();

    for (name, account) in new_accounts {
        // TODO once drain filter is stabilized move to it https://github.com/rust-lang/rust/issues/43244
        let collected_emails = emails
            .iter()
            .filter(|e| e.name == name)
            .map(|e| {
                let email_type = EmailType::from_str(&e.address).unwrap_or_else(|_|{
                    warn!("Failed to parse email type for email {email} for account {name}. Defaulting to Alias", email = e.address, name = name);
                    EmailType::Alias
                });
                (e.address.clone(), email_type)
            })
            .collect::<Vec<(String, EmailType)>>();

        let id = match AccountEntity::insert(account)
            .on_conflict(
                OnConflict::column(entities::account::Column::Username)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(database)
            .await
        {
            Ok(ok) => ok.last_insert_id,
            Err(err) => {
                error!("Failed to insert account {}. Error {err}", name);
                has_errors = true;
                continue;
            }
        };
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

            if let Err(error) = EmailEntity::insert(email).exec(database).await {
                error!("Failed to Insert Email {error} for Account {name}");
                has_errors = true;
            }
        }
    }
    Ok(has_errors)
}
