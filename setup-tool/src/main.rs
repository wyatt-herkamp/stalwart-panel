use std::{borrow::Cow, fs::read_to_string, mem, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand, ValueEnum};
use entities::{
    account::AccountType,
    groups::{ActiveModel, GroupPermissions},
    now, AccountEntity, ActiveAccountModel, GroupEntity,
};
use log::{debug, error, info};
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    sea_query::OnConflict, ActiveValue, ConnectOptions, DatabaseConnection, EntityTrait,
};
use thiserror::Error;
use toml_edit::{Document, TomlError};
use utils::{
    config::{
        Database, EmailEncryption, EmailSetting, MysqlSettings, PostgresSettings, Settings,
        StalwartManagerConfig,
    },
    database::{
        password::{PasswordErrors, PasswordType},
        Password,
    },
    stalwart_manager::ManagerConfig,
};

use crate::config_updater::update_config;
#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to hash password {0}")]
    PasswordHashError(#[from] PasswordErrors),
    #[error("Failed to connect to database {0}")]
    SeaORMError(#[from] sea_orm::error::DbErr),
    #[error("Failed to connect to database {0}")]
    SQLXError(#[from] sqlx::Error),
    #[error(transparent)]
    IOErr(#[from] std::io::Error),
    #[error("Failed to Serialize Config {0}. This is a bug Please report it")]
    TomlSerializeError(#[from] toml::ser::Error),
    #[error("Failed to Deserialize Config {0}. Error {1}")]
    TomlDeserializeError(PathBuf, TomlError),
}
mod ask_questions;
mod config_updater;
mod database_importer;

#[derive(ValueEnum, Clone)]
pub enum DatabaseType {
    Mysql,
    Postgres,
}
#[derive(Parser)]
struct Command {
    // The Stalwart Config File
    #[clap(long)]
    stalwart_config: PathBuf,
    #[clap(subcommand)]
    subcommand: Option<Commands>,

    #[clap(long, default_value = "false")]
    skip_import: bool,
    // Prevent Questions from being asked during the setup
    #[clap(long, default_value = "false")]
    no_questions_asked: bool,
    // All uses imported will require a new password
    #[clap(long, default_value = "false")]
    require_password_changes_on_all_users: bool,
    // Should the database Migration be a fresh install
    #[clap(long, default_value = "true")]
    fresh_database: bool,
}
#[derive(Subcommand)]
enum Commands {
    Auto {
        // The Database Type for the new database
        #[clap(long)]
        database_type: DatabaseType,
        // The Database Host for the new database
        #[clap(long)]
        database_host: String,
        // The Database User for the new database
        #[clap(long)]
        database_user: String,
        // The Database Password for the new database
        #[clap(long)]
        database_password: String,
        // The Database Name for the new database
        #[clap(long)]
        database_name: String,
    },
}

impl Commands {
    pub fn get_database_config(&mut self) -> Database {
        match self {
            Commands::Auto {
                database_type,
                database_host,
                database_user,
                database_password,
                database_name,
                ..
            } => match database_type {
                DatabaseType::Mysql => Database::Mysql(MysqlSettings {
                    user: mem::take(database_user),
                    password: mem::take(database_password),
                    host: mem::take(database_host),
                    database: mem::take(database_name),
                }),
                DatabaseType::Postgres => Database::Postgres(PostgresSettings {
                    user: mem::take(database_user),
                    password: mem::take(database_password),
                    host: mem::take(database_host),
                    database: mem::take(database_name),
                }),
            },
        }
    }
}
#[tokio::main]
async fn main() -> Result<ExitCode, Error> {
    let Command {
        stalwart_config: stalwart_config_path,
        subcommand,
        skip_import,
        no_questions_asked,
        require_password_changes_on_all_users,
        fresh_database,
    } = Command::parse();
    let database_config = match subcommand {
        None => ask_questions::get_database_config(),
        Some(mut sub_command) => Some(sub_command.get_database_config()),
    };
    let Some(database_config) = database_config else {
        error!("No Database Config Provided");
        return Ok(ExitCode::FAILURE);
    };
    sqlx::any::install_default_drivers();
    let toml_content = read_to_string(&stalwart_config_path)?;

    let mut database_connection =
        sea_orm::Database::connect(ConnectOptions::new(database_config.to_string())).await?;

    let stalwart_config: Document = toml_content
        .parse()
        .map_err(|err| Error::TomlDeserializeError(stalwart_config_path.clone(), err))?;

    let super_user_name = stalwart_config["directory"]["sql"]["options"]["superuser-group"]
        .as_str()
        .unwrap_or_else(|| {
            info!("Missing superuser-group in stalwart config. Defaulting to `superuser`");
            "superuser"
        });
    setup_database(&mut database_connection, super_user_name, fresh_database).await;

    let password: Cow<'static, str> = if !skip_import {
        let old_database = stalwart_config["directory"]["sql"]["address"].as_str();
        if let Some(value) = old_database {
            info!("Importing Old Database");
            if let Err(err) = database_importer::import_database(
                &mut database_connection,
                value,
                &super_user_name,
                no_questions_asked,
                require_password_changes_on_all_users,
            )
            .await
            {
                error!("Unable to import old database. Error {}", err);
            }
            Cow::Borrowed("<Please Provide Password>")
        } else {
            info!("Skipping Import. Unable to find old database");
            Cow::Borrowed("<Please Provide Password>")
        }
    } else {
        info!("Skipping Import. Creating Default Account");
        create_default_account(&mut database_connection)
            .await
            .map(Cow::Owned)?
    };

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
        password: password.into_owned(),
        host: stalwart_config["server"]["hostname"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or("localhost".to_string()),
        encryption: EmailEncryption::StartTLS,
        from: format!("Stalwart Panel<panel@{}>", main_domain),
        reply_to: None,
    };
    update_config(&database_config, stalwart_config, &stalwart_config_path);

    info!("Stalwart has been configured. Creating the panel config");

    let config = Settings::new(database_config, postmaster_address, email);

    let config = toml::to_string_pretty(&config)?;

    let config_file = PathBuf::from("stalwart-panel.toml");
    if config_file.exists() {
        std::fs::remove_file(&config_file)?;
    }
    std::fs::write(&config_file, &config)?;

    let stalwart_manager_config = ManagerConfig {
        stalwart_config: stalwart_config_path,
        ..StalwartManagerConfig::default()
    };

    let stalwart_manager_config = toml::to_string_pretty(&stalwart_manager_config)?;

    let stalwart_manager_config_file = PathBuf::from("stalwart-manager.toml");

    if stalwart_manager_config_file.exists() {
        std::fs::remove_file(&stalwart_manager_config_file)?;
    }

    std::fs::write(&stalwart_manager_config_file, &stalwart_manager_config)?;

    info!("Stalwart Panel has been configured. Please double check stalwart-panel.toml and then run stalwart-panel");
    return Ok(ExitCode::FAILURE);
}

async fn setup_database(
    database_connection: &DatabaseConnection,
    super_user_name: &str,
    fresh_database: bool,
) {
    if fresh_database {
        info!("Fresh Database Flag Set. All old data will be deleted");
        Migrator::fresh(database_connection)
            .await
            .expect("Failed to run migrations");
    } else {
        Migrator::up(database_connection, None)
            .await
            .expect("Failed to run migrations");
    }

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

async fn create_default_account(
    database_connection: &mut DatabaseConnection,
) -> Result<String, Error> {
    let (password_hashed, password) = Password::create_password(PasswordType::Argon2)?;
    let postmaster = ActiveAccountModel {
        id: ActiveValue::Set(1),
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

    Ok(password)
}
