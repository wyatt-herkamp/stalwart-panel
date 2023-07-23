pub mod api;
pub mod auth;
pub mod email_service;
pub mod error;
pub mod frontend;

use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{App, HttpServer, Scope};
use chrono::Duration;
use clap::Parser;
use parking_lot::Mutex;
use rustls::{Certificate, PrivateKey, ServerConfig as RustlsServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use sea_orm::{ConnectOptions, Database};
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use tokio::fs::read_to_string;
use utils::config::Settings;
use utils::stalwart_manager::StalwartManager;

use crate::auth::middleware::HandleSession;
use crate::auth::password_reset::PasswordResetManager;
use crate::auth::session::SessionManager;
use crate::email_service::EmailService;
pub use error::WebsiteError as Error;

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Parser)]
struct Command {
    #[clap(short, long)]
    config: PathBuf,
    #[clap(short, long)]
    stalwart_manager: PathBuf,
}
pub type DatabaseConnection = Data<sea_orm::DatabaseConnection>;
/// Mutex's are slightly faster than RwLocks and we don't really need to have multiple readers
/// This could be changed in the future if we need to have multiple readers
pub type SlalwartManager = Data<Mutex<StalwartManager>>;
#[actix_web::main]
async fn main() -> io::Result<()> {
    let command = Command::parse();

    let config = read_to_string(command.config).await?;

    let server_config: Settings = toml::from_str(&config).expect("Failed to parse config");

    let database = Database::connect(ConnectOptions::new(server_config.database.to_string()))
        .await
        .map(Data::new)
        .expect("Failed to connect to database");

    let stalwart_manager = StalwartManager::new(command.stalwart_manager)
        .map(|v| Data::new(Mutex::new(v)))
        .expect("Failed to create Stalwart Manager");

    let session_manager = SessionManager::new(PathBuf::new().join("sessions.redb"))
        .map(Data::new)
        .expect("Failed to create session manager");

    SessionManager::start_cleaner(session_manager.clone().into_inner(), Duration::hours(1));

    let email = EmailService::start(server_config.email)
        .await
        .expect("Failed to start email service")
        .map(Data::new)
        .expect("Failed to start email service");

    let password_reset = Data::new(PasswordResetManager {
        email_access: email.clone().into_inner(),
        requests: Default::default(),
    });
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method()
            .supports_credentials();
        App::new()
            .app_data(database.clone())
            .app_data(stalwart_manager.clone())
            .app_data(session_manager.clone())
            .app_data(email.clone())
            .app_data(password_reset.clone())
            .wrap(cors)
            .service(
                Scope::new("/api")
                    .wrap(HandleSession(session_manager.clone()))
                    .service(Scope::new("/accounts").configure(api::accounts::init)),
            )
            .service(
                Scope::new("/frontend")
                    .service(Scope::new("/backend").configure(frontend::api::init)),
            )
    });

    let server = if let Some(tls) = server_config.tls {
        let mut cert_file = BufReader::new(File::open(tls.certificate_chain)?);
        let mut key_file = BufReader::new(File::open(tls.private_key)?);

        let cert_chain = certs(&mut cert_file)
            .expect("server certificate file error")
            .into_iter()
            .map(Certificate)
            .collect();
        let mut keys: Vec<PrivateKey> = pkcs8_private_keys(&mut key_file)
            .expect("server private key file error")
            .into_iter()
            .map(PrivateKey)
            .collect();

        let config = RustlsServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, keys.remove(0))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        server.bind_rustls(server_config.bind_address, config)?
    } else {
        server.bind(server_config.bind_address)?
    };
    server.run().await?;
    Ok(())
}
