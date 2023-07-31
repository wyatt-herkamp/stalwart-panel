use std::fs::read_to_string;
use std::path::PathBuf;

use toml_edit::{Document, Item};

use utils::config::Database;
use utils::stalwart_config::sql::{SQLColumns, SQLQuery};

#[test]
pub fn test_config_update() {
    let current_dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .parent()
        .unwrap()
        .to_path_buf();
    let file = current_dir.join("test_data").join("stalwart-config.toml");
    assert!(file.exists(), "Test file does not exist {}", file.display());
    let copy = current_dir
        .join("test_data")
        .join("stalwart-config-test.toml");
    if copy.exists() {
        std::fs::remove_file(&copy).expect("Failed to remove test file");
    }
    std::fs::copy(&file, &copy).expect("Failed to copy file");

    drop(file);
    let toml_content = read_to_string(&copy).expect("Failed to read stalwart config file");

    let document: Document = toml_content
        .parse()
        .expect("Failed to parse stalwart config file");

    let database = Database::test();

    update_config(&database, document, &copy);
}

pub(crate) fn update_config(
    database_config: &Database,
    mut stalwart_config: Document,
    file: &PathBuf,
) {
    stalwart_config["directory"]["sql"]["address"] =
        Item::Value(database_config.to_string().into());

    let queries = match &database_config {
        Database::Mysql(_) => SQLQuery::new_mysql(),
        Database::Postgres(_) => SQLQuery::new_postgres(),
    };
    stalwart_config["directory"]["sql"]["queries"] = queries.into();
    stalwart_config["directory"]["sql"]["columns"] = SQLColumns::default().into();

    // Backup the old config file
    let backup_file = file.with_extension("bak.toml");
    if backup_file.exists() {
        std::fs::remove_file(&backup_file).expect("Failed to remove old backup file");
    }
    std::fs::rename(&file, &backup_file).expect("Failed to backup stalwart config file");
    let content = stalwart_config.to_string();
    std::fs::write(file, &content).expect("Failed to write new stalwart config file");
}
