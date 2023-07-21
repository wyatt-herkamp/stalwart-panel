use crate::config::StalwartManagerConfig;
use crate::stalwart_manager::app_connector::AppConnection;
use log::warn;
use std::path::PathBuf;
use thiserror::Error;
use toml_edit::{Array, Document, Formatted, Item, Table, Value};

pub mod app_connector;
#[cfg(not(target_os = "linux"))]
pub type AppConnectionImpl = app_connector::none::NoneConnection;
#[cfg(target_os = "linux")]
pub type AppConnectionImpl = app_connector::linux_connection::LinuxConnection;

pub type ManagerConfig = StalwartManagerConfig<<AppConnectionImpl as AppConnection>::Config>;
#[derive(Debug, Error)]
pub enum StalwartError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    TomlEdit(#[from] toml_edit::TomlError),
    /// It wouldn't come up after the change and the revert
    #[error("Stalwart cannot start")]
    StalwartCannotStart,
    /// It wouldn't come up after the change
    #[error("Stalwart did not start after change")]
    StalwartDidNotStartAfterChange,
}
#[derive(Debug)]
pub struct StalwartManager {
    pub app_connection: AppConnectionImpl,
    pub stalwart_config: ManagerConfig,
    pub stalwart_config_location: PathBuf,
    pub config: Document,
}

impl StalwartManager {
    pub fn new(stalwart_config: PathBuf) -> Result<Self, StalwartError> {
        let config = std::fs::read_to_string(&stalwart_config).expect("Failed to read config");
        let manager_config: ManagerConfig = toml::from_str(&config)?;

        let app_connection = AppConnectionImpl::new(manager_config.manager_config.clone());

        let stalwart_config_location = manager_config.stalwart_config.clone();

        let config = std::fs::read_to_string(&stalwart_config_location)?;
        let config = config.parse::<Document>()?;

        Ok(Self {
            app_connection,
            stalwart_config: manager_config,
            stalwart_config_location,
            config,
        })
    }
    pub fn backup(&self) -> Result<(), StalwartError> {
        let backup_location = self
            .stalwart_config
            .stalwart_config
            .with_extension("bak.toml");
        if backup_location.exists() {
            std::fs::remove_file(&backup_location)?;
        }
        std::fs::copy(&self.stalwart_config.stalwart_config, &backup_location)
            .map(|_| ())
            .map_err(|e| StalwartError::from(e))
    }
    pub fn revert_backup(&mut self) -> Result<(), StalwartError> {
        let backup_location = self
            .stalwart_config
            .stalwart_config
            .with_extension("bak.toml");
        if backup_location.exists() {
            std::fs::copy(&backup_location, &self.stalwart_config.stalwart_config)
                .map(|_| ())
                .map_err(|e| StalwartError::from(e))?;
            let config = std::fs::read_to_string(&self.stalwart_config.stalwart_config)?;
            self.config = config.parse::<Document>()?;

            std::fs::remove_file(&backup_location)?;
        } else {
            warn!("No backup found");
        }
        Ok(())
    }

    pub fn enable_tracing_to_panel(
        &mut self,
        panel_url: &str,
        panel_key: &str,
        level: &str,
    ) -> Result<(), StalwartError> {
        let endpoint = format!("{panel_url}/api/trace/receiver");
        let header = format!("Authorization: bearer {panel_key}");
        let mut tracing_table = Table::new();
        tracing_table["method"] =
            Item::Value(Value::String(Formatted::new("open-telemetry".into())));
        tracing_table["transport"] = Item::Value(Value::String(Formatted::new("http".into())));
        tracing_table["endpoint"] = Item::Value(Value::String(Formatted::new(endpoint)));
        let mut headers = Array::new();
        headers.push(Value::String(Formatted::new(header)));
        tracing_table["headers"] = Item::Value(Value::Array(headers));
        tracing_table["level"] = Item::Value(Value::String(Formatted::new(level.into())));

        self.config
            .insert("global.tracing", Item::Table(tracing_table));

        // Save the config

        let config = self.config.to_string();
        self.backup()?;

        std::fs::write(&self.stalwart_config.stalwart_config, config)?;
        if self.app_connection.restart().is_err() {
            warn!("Failed to restart app reverting changes");
            self.revert_backup()?;
            return if self.app_connection.restart().is_err() {
                Err(StalwartError::StalwartCannotStart)
            } else {
                Err(StalwartError::StalwartDidNotStartAfterChange)
            };
        }
        Ok(())
    }
}
