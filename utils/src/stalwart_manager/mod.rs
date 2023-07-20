use log::warn;
use thiserror::Error;
use toml_edit::{Array, Document, Formatted, Item, Table, Value};
use crate::stalwart_manager::app_connector::AppConnection;

mod app_connector;
#[cfg(not(target_os = "linux"))]
pub type AppConnectionImpl = app_connector::none::NoneConnection;
#[cfg(target_os = "linux")]
pub type AppConnectionImpl = app_connector::linux_connection::LinuxConnection;
#[derive(Debug, Error)]
pub enum StalwartError {
    IO(#[from] std::io::Error),
    /// It wouldn't come up after the change and the revert
    StalwartCannotStart,
    /// It wouldn't come up after the change
    StalwartDidNotStartAfterChange
}
#[derive(Debug)]
pub struct StalwartManager {
    pub app_connection: AppConnectionImpl,
    pub config_location: std::path::PathBuf,
    pub config: Document,
}

impl StalwartManager {
    pub fn backup(&self) -> Result<(), StalwartError> {
        let backup_location = self.config_location.with_extension("bak.toml");
        if backup_location.exists() {
            std::fs::remove_file(&backup_location)?;
        }
        std::fs::copy(&self.config_location, &backup_location)
            .map(|_| ())
            .map_err(|e| StalwartError::from(e))
    }
    pub fn revert_backup(&mut self)  -> Result<(), StalwartError>{

        let backup_location = self.config_location.with_extension("bak.toml");
        if backup_location.exists() {
            std::fs::copy(&backup_location, &self.config_location)
                .map(|_| ())
                .map_err(|e| StalwartError::from(e))?;
            std::fs::remove_file(&backup_location)?;
        }else{
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

        std::fs::write(&self.config_location, config)?;
        if self.app_connection.restart().is_err(){
            warn!("Failed to restart app reverting changes");
            self.revert_backup()?;
            return if self.app_connection.restart().is_err() {
                Err(StalwartError::StalwartCannotStart)
            } else {
                Err(StalwartError::StalwartDidNotStartAfterChange)
            }
        }
        Ok(())


    }
}
