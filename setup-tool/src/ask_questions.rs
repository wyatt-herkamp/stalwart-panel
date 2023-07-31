use crate::DatabaseType;
use inquire::{Select, Text};
use utils::config::Database;

pub fn get_database_config() -> Option<Database> {
    let database_type = Select::new("Database Type", vec!["Mysql", "Postgres"])
        .prompt()
        .map(|x| match x {
            "Mysql" => DatabaseType::Mysql,
            "Postgres" => DatabaseType::Postgres,
            _ => unreachable!(),
        })
        .ok()?;

    let user = Text::new("Database User")
        .with_help_message("The user to connect to the database with")
        .prompt()
        .ok()?;

    let password = Text::new("Database Password")
        .with_help_message("The password to connect to the database with")
        .prompt()
        .ok()?;

    let host = Text::new("Database Host")
        .with_help_message("The host to connect to the database with")
        .with_default("localhost:5432")
        .prompt()
        .ok()?;

    let database = Text::new("Database Name")
        .with_help_message("The name of the database to connect to")
        .with_default("stalwart-panel")
        .prompt()
        .ok()?;
    match database_type {
        DatabaseType::Mysql => Some(Database::Mysql(utils::config::MysqlSettings {
            user,
            password,
            host,
            database,
        })),
        DatabaseType::Postgres => Some(Database::Postgres(utils::config::PostgresSettings {
            user,
            password,
            host,
            database,
        })),
    }
}
