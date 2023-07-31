mod random_users;

use clap::{Parser, Subcommand};
use entities::{account, AccountEntity, EmailEntity};
use sea_orm::prelude::*;
use sea_orm::{ConnectOptions, Database};
use std::path::PathBuf;
use utils::config::Settings;

#[derive(Debug, Parser)]
pub struct Command {
    pub panel_config: PathBuf,
    #[clap(subcommand)]
    pub subcommand: Commands,
    #[clap(long, default_value = "example.com")]
    pub test_domain: String,
}
#[derive(Debug, Subcommand)]
pub enum Commands {
    GenerateRandomUsers(random_users::RandomUsersCommand),
    DeleteRandomUsers,
}

#[tokio::main]
async fn main() {
    let command = Command::parse();

    let panel_config = std::fs::read_to_string(command.panel_config).unwrap();

    let panel_config: Settings = toml::from_str(&panel_config).unwrap();
    let database = Database::connect(ConnectOptions::new(panel_config.database.to_string()))
        .await
        .expect("Failed to connect to database");

    match command.subcommand {
        Commands::GenerateRandomUsers(sub_command) => {
            sub_command
                .run(database, command.test_domain, panel_config.default_group)
                .await;
        }
        Commands::DeleteRandomUsers => {
            account::Entity::delete_many()
                .filter(
                    account::Column::Description
                        .contains("source: randomuser.me")
                        .and(account::Column::GroupId.eq(panel_config.default_group)),
                )
                .exec(&database)
                .await
                .expect("Failed to delete users");
        }
    }
}
