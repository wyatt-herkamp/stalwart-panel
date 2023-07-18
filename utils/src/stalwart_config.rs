use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;


use serde::{Deserialize, Serialize};
use toml::Value;

pub type Catchall = HashMap<String, Value>;
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StalwartConfig {
    pub management: Management,
    pub directory: Directory,
    #[serde(flatten)]
    pub catchall: Catchall,
}
#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct Management{
    pub directory: String,
    #[serde(flatten)]
    pub catchall: Catchall,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Directory{
    pub sql: SQLSettings,
    #[serde(flatten)]
    pub catchall: Catchall
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SQLSettings{
    pub address: String,
    #[serde(flatten)]
    pub catchall: Catchall,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SQLQuery{
    pub name: String,

}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SQLColumns{
    pub name: String,
    pub description: String,
    pub secret: String,
    pub email: String,
    pub quota: String,
    #[serde(rename = "type")]
    pub email_type: String,
}