use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use chrono::Duration;
use serde::{Deserialize, Serialize};

use crate::database::password::PasswordType;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "settings")]
pub enum Database {
    Mysql(MysqlSettings),
    Postgres(PostgresSettings),
}
impl Database {
    pub fn test() -> Self {
        Database::Postgres(PostgresSettings {
            user: "postgres".to_string(),
            password: "postgres".to_string(),
            host: "localhost:5432".to_string(),
            database: "stalwart-panel-test".to_string(),
        })
    }
}
impl Display for Database {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Database::Mysql(mysql) => write!(f, "{}", mysql),
            Database::Postgres(postgres) => write!(f, "{}", postgres),
        }
    }
}
impl Display for MysqlSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "mysql://{}:{}@{}/{}",
            self.user, self.password, self.host, self.database
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MysqlSettings {
    pub user: String,
    pub password: String,
    pub host: String,
    pub database: String,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostgresSettings {
    pub user: String,
    pub password: String,
    pub host: String,
    pub database: String,
}

impl Display for PostgresSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "postgres://{}:{}@{}/{}",
            self.user, self.password, self.host, self.database
        )
    }
}
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum EmailEncryption {
    #[default]
    NONE,
    StartTLS,
    TLS,
}
/// Yes the email software management software needs email settings
///
/// This is for sending reset password emails and any other emails.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmailSetting {
    pub username: String,
    pub password: String,
    pub host: String,
    pub encryption: EmailEncryption,
    pub from: String,
    pub reply_to: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SessionManager {
    #[serde(with = "crate::duration_serde::as_seconds")]
    pub lifespan: Duration,
    #[serde(with = "crate::duration_serde::as_seconds")]
    pub cleanup_interval: Duration,
    pub dev: bool,
    pub database_location: PathBuf,
}
impl Default for SessionManager {
    fn default() -> Self {
        Self {
            lifespan: Duration::days(1),
            cleanup_interval: Duration::hours(1),
            dev: false,
            database_location: PathBuf::from("sessions.redb"),
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PasswordReset {
    // If this is 0 then it will never force a password reset
    #[serde(with = "crate::duration_serde::as_days")]
    pub how_often_to_force_reset: Duration,
    #[serde(with = "crate::duration_serde::as_seconds")]
    pub reset_service_interval: Duration,
}
impl Default for PasswordReset {
    fn default() -> Self {
        Self {
            how_often_to_force_reset: Duration::days(0),
            reset_service_interval: Duration::days(1),
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub bind_address: String,
    #[serde(default = "default_workers")]
    pub number_of_workers: usize,
    pub database: Database,
    pub tls: Option<TlsConfig>,
    pub email: EmailSetting,
    pub postmaster_address: String,
    pub default_group: i64,
    pub root_group: i64,
    #[serde(default)]
    pub password_hash_for_new_passwords: PasswordType,
    #[serde(default)]
    pub require_password_reset: PasswordReset,
    #[serde(default)]
    pub session_manager: SessionManager,
    /// This is ignored if the tls config is set
    #[serde(default)]
    pub is_https: bool,
}
fn default_workers() -> usize {
    2
}
impl Settings {
    pub fn new(database: Database, postmaster_address: String, email: EmailSetting) -> Self {
        Self {
            bind_address: "0.0.0.0:5312".to_string(),
            number_of_workers: 2,
            database,
            tls: None,
            email,
            postmaster_address,
            default_group: 1,
            root_group: 2,
            password_hash_for_new_passwords: Default::default(),
            require_password_reset: Default::default(),
            session_manager: Default::default(),
            is_https: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TlsConfig {
    pub private_key: PathBuf,
    pub certificate_chain: PathBuf,
}

/// This is in a separate file because it gets edited during runtime
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StalwartManagerConfig<Config> {
    // Used for Adding Domains
    pub stalwart_config: PathBuf,
    // Restart Stalwart Command
    pub tracing_enabled: bool,

    pub manager_config: Config,
}
impl<Config: Default> Default for StalwartManagerConfig<Config> {
    fn default() -> Self {
        Self {
            stalwart_config: PathBuf::new(),
            tracing_enabled: false,
            manager_config: Config::default(),
        }
    }
}
