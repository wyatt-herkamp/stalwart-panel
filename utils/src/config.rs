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

/// Yes the email software management software needs email settings
///
/// This is for sending reset password emails and any other emails.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmailSetting {
    pub username: String,
    pub password: String,
    pub host: String,
    pub encryption: String,
    pub from: String,
    pub port: u16,
}

impl Default for EmailSetting {
    fn default() -> Self {
        EmailSetting {
            username: "no-reply@example.com".to_string(),
            password: "".to_string(),
            host: "example.com".to_string(),
            encryption: "TLS".to_string(),
            from: "no-reply@example.com".to_string(),
            port: 587,
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub database: Database,
    pub email: EmailSetting,
    pub postmaster_address: String,
    pub default_group: i64,
    pub root_group: i64,
    // Used for Adding Domains
    pub stalwart_config: PathBuf,
    // Restart Stalwart Command
    pub restart_stalwart_command: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self{
            database: Database::None,
            email: Default::default(),
            postmaster_address: "".to_string(),
            default_group: 0,
            root_group: 1,
            stalwart_config: PathBuf::new(),
            #[cfg(not(target_os = "linux"))]
            // TODO add default for windows
            restart_stalwart_command: "".to_string(),
            #[cfg(target_os = "linux")]
            restart_stalwart_command: "/usr/bin/systemctl restart stalwart-mail.service".to_string(),
        }
    }
}
