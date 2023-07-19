use std::borrow::Cow;

use crate::config::Database;
use serde::{Deserialize, Serialize};
use toml_edit::{Item, Value};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SQLSettings {
    pub address: String,
    #[serde(rename = "type")]
    pub directory_type: Cow<'static, str>,
}

impl SQLSettings {
    pub fn new(database: &Database) -> Self {
        Self {
            address: database.to_string(),
            directory_type: "sql".into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SQLOptions {
    pub max_connections: u32,
    pub min_connections: u32,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: String,
}
fn default_idle_timeout() -> String {
    "10m".into()
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SQLCache {
    pub entries: u32,
    pub ttl: TTL,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TTL {
    pub positive: String,
    pub negative: String,
}
impl Default for TTL {
    fn default() -> Self {
        Self {
            positive: "1h".into(),
            negative: "10m".into(),
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SQLQuery {
    pub name: Cow<'static, str>,
    pub members: Cow<'static, str>,
    pub recipients: Cow<'static, str>,
    pub emails: Cow<'static, str>,
    pub verify: Cow<'static, str>,
    pub expand: Cow<'static, str>,
    pub domains: Cow<'static, str>,
}

impl SQLQuery {
    /// This is the queries that are used in Stalwart to use this panels database for Postgres
    pub fn new_postgres() -> Self {
        Self{
            name: "SELECT username, type, password, description, quota FROM accounts WHERE name = $1 AND active = true".into(),
            members: "SELECT g.group_name FROM groups AS g INNER JOIN accounts as a ON g.id = a.group_id AND a.username = $1".into(),
            recipients: "SELECT a.username FROM accounts as a INNER JOIN emails as e ON a.id = e.account AND e.email = $1".into(),
            emails: "SELECT e.email FROM emails as e INNER JOIN accounts as a ON e.account = a.id AND a.username = $1".into(),
            verify: "SELECT email FROM emails WHERE email LIKE '%' || $1 || '%' AND type = 'primary' ORDER BY email LIMIT 5".into(),
            expand: "SELECT p.email FROM emails AS p JOIN emails AS l ON p.name = l.name WHERE p.type = 'primary' AND l.email = $1 AND l.type = 'list' ORDER BY p.email LIMIT 50".into(),
            domains: "SELECT 1 FROM emails WHERE email LIKE '%@' || $1 LIMIT 1".into(),
        }
    }
    /// This is the queries that are used in Stalwart to use this panels database for MySQL.
    pub fn new_mysql() -> Self {
        Self{
            name: "SELECT username, type, password, description, quota FROM accounts WHERE name = ? AND active = true".into(),
            members: "SELECT g.group_name FROM groups AS g INNER JOIN accounts as a ON g.id = a.group_id AND a.username = ?".into(),
            recipients: "SELECT a.username FROM accounts as a INNER JOIN emails as e ON a.id = e.account AND e.email = ?".into(),
            emails: "SELECT e.email FROM emails as e INNER JOIN accounts as a ON e.account = a.id AND a.username = ?".into(),
            verify: "SELECT email FROM emails WHERE email LIKE '%' || ? || '%' AND type = 'primary' ORDER BY email LIMIT 5".into(),
            expand: "SELECT p.email FROM emails AS p JOIN emails AS l ON p.name = l.name WHERE p.type = 'primary' AND l.email = ? AND l.type = 'list' ORDER BY p.email LIMIT 50".into(),
            domains: "SELECT 1 FROM emails WHERE email LIKE '%@' || ? LIMIT 1".into(),
        }
    }
}
impl Into<Item> for SQLQuery {
    fn into(self) -> Item {
        let mut table = toml_edit::Table::new();
        table["name"] = Item::Value(Value::from(self.name.into_owned()));
        table["members"] = Item::Value(Value::from(self.members.into_owned()));
        table["recipients"] = Item::Value(Value::from(self.recipients.into_owned()));
        table["emails"] = Item::Value(Value::from(self.emails.into_owned()));
        table["verify"] = Item::Value(Value::from(self.verify.into_owned()));
        table["expand"] = Item::Value(Value::from(self.expand.into_owned()));
        table["domains"] = Item::Value(Value::from(self.domains.into_owned()));
        Item::Table(table)
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SQLColumns {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub secret: Cow<'static, str>,
    pub email: Cow<'static, str>,
    pub quota: Cow<'static, str>,
    #[serde(rename = "type")]
    pub email_type: Cow<'static, str>,
}

impl Default for SQLColumns {
    /// This is the columns that are used in Stalwart to use this panels database.
    fn default() -> Self {
        Self {
            name: "username".into(),
            description: "description".into(),
            secret: "password".into(),
            email: "email".into(),
            quota: "quota".into(),
            email_type: "account_type".into(),
        }
    }
}

impl Into<Item> for SQLColumns {
    fn into(self) -> Item {
        let mut table = toml_edit::Table::new();
        table["name"] = Item::Value(Value::from(self.name.into_owned()));
        table["description"] = Item::Value(Value::from(self.description.into_owned()));
        table["secret"] = Item::Value(Value::from(self.secret.into_owned()));
        table["email"] = Item::Value(Value::from(self.email.into_owned()));
        table["quota"] = Item::Value(Value::from(self.quota.into_owned()));
        table["type"] = Item::Value(Value::from(self.email_type.into_owned()));
        Item::Table(table)
    }
}
