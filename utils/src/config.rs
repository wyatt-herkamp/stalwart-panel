use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "settings")]
pub enum Database {
    Mysql(MysqlSettings),
    Postgres(PostgresSettings),
    None,
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
impl Default for Database {
    fn default() -> Self {
        Database::None
    }
}
impl Display for Database {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Database::Mysql(mysql) => write!(f, "{}", mysql),
            Database::Postgres(postgres) => write!(f, "{}", postgres),
            Database::None => panic!("No Database Configured"),
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

impl Default for EmailSetting {
    fn default() -> Self {
        EmailSetting {
            username: "no-reply@example.com".to_string(),
            password: "".to_string(),
            host: "example.com:587".to_string(),
            encryption: EmailEncryption::TLS,
            from: "no-reply@example.com".to_string(),
            reply_to: None,
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub bind_address: String,
    pub database: Database,
    pub tls: Option<TlsConfig>,
    pub email: EmailSetting,
    pub postmaster_address: String,
    pub default_group: i64,
    pub root_group: i64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:5312".to_string(),
            database: Database::None,
            tls: None,
            email: Default::default(),
            postmaster_address: "".to_string(),
            default_group: 1,
            root_group: 2,
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
