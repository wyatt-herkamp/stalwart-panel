use std::collections::HashMap;
use std::str::FromStr;

use inquire::Confirm;
use log::{debug, info, warn};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use sqlx::{Connection, SqliteConnection};

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
pub(crate) async fn import_database(
    database: &mut DatabaseConnection,
    old_database: &str,
    superuser: &str,
    no_questions_asked: bool,
    require_password_changes_on_all_users: bool,
) {
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
            let mut password = Password::new_hashed(password);
            match password.hash_type() {
                PasswordType::PlainText => {
                    if no_questions_asked{
                        if let Ok(value) = Confirm::new(
                            r#"Password is in plain text. Would you like to hash it using Argon2. 
                            If not you will need to reset your password to access the Stalwart Panel"#
                        )
                            .with_default(false)
                            .prompt()
                        {
                            if value {
                                password
                                    .hash(PasswordType::Argon2)
                                    .expect("Failed to hash password");
                            }
                        };
                    }else{
                        warn!(
                            r#"Account {username} has a plain text password.
                             This user will not able to log into Stalwart Panel Until it is updated"#
                        )
                    }

                }
                PasswordType::None => {
                    warn!(
                        r#"Account {username} has an unsupported Password Hash for Stalwart Panel.
                Accessing your Email account will continue to work,
                However, you will need to reset your password to access the Stalwart Panel"#
                    );
                }
                _ => {}
            }
            (
                username.clone(),
                ActiveAccountModel {
                    id: Default::default(),
                    name: ActiveValue::Set(username.clone()),
                    username: ActiveValue::Set(username),
                    description: ActiveValue::Set(description),
                    group_id: ActiveValue::Set(group_id),
                    password: ActiveValue::Set(password),
                    require_password_change: ActiveValue::Set(require_password_changes_on_all_users),
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
