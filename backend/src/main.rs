pub mod api;
pub mod auth;
pub mod email_service;
pub mod error;
pub mod frontend;
pub mod headers;

use std::{fs::File, io, io::BufReader, path::PathBuf};

use actix_cors::Cors;
use actix_web::{web::Data, App, HttpServer, Scope};
use clap::Parser;
pub use error::WebsiteError as Error;
use parking_lot::Mutex;
use sea_orm::{ConnectOptions, Database};
use tokio::fs::read_to_string;
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::prelude::*;
use utils::{
    config::{Settings, TlsConfig},
    database::password::PasswordType,
    stalwart_manager::StalwartManager,
};

use crate::{
    auth::{
        middleware::HandleSession, password_reset::PasswordResetManager, session::SessionManager,
    },
    email_service::EmailService,
};

#[cfg(not(any(feature = "rust-tls", feature = "native-tls")))]
compile_error!("Feature 'rust-tls' or 'native-tls' must be enabled to use stalwart-panel as lettre requires one of these features to be enabled.");
#[cfg(all(feature = "rust-tls", feature = "native-tls"))]
compile_error!(
    "Feature 'rust-tls' and 'native-tls' cannot be enabled at the same time. Please choose one."
);
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Parser)]
struct Command {
    /// The stalwart-panel config file
    #[clap(short, long, default_value = "stalwart-panel.toml")]
    config: PathBuf,
    // Comments will be destroyed by TOML
    #[clap(long, default_value = "false")]
    add_defaults_to_config: bool,
}

pub type DatabaseConnection = Data<sea_orm::DatabaseConnection>;
/// Mutex's are slightly faster than RwLocks and we don't really need to have multiple readers
/// This could be changed in the future if we need to have multiple readers
pub type SlalwartManager = Data<Mutex<StalwartManager>>;
#[derive(Clone)]
pub struct SharedConfig {
    password_hash: PasswordType,
    https: bool,
}
#[actix_web::main]
async fn main() -> io::Result<()> {
    let collector = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    // filter spans/events with level TRACE or higher.
    // build but do not install the subscriber.

    collector.init();

    let command = Command::parse();

    let config = read_to_string(&command.config).await?;

    let server_config: Settings = toml::from_str(&config).expect("Failed to parse config");

    if command.add_defaults_to_config {
        info!("`--add-defaults-to-config` was passed, Config will be saved with missing values filled in. This will remove all comments from the config file.");
        let config = toml::to_string_pretty(&server_config).expect("Failed to serialize config");
        std::fs::write(command.config, config).expect("Failed to write config");
    }
    let Settings {
        bind_address,
        number_of_workers,
        database,
        tls,
        email,
        password_hash_for_new_passwords,
        session_manager,
        is_https,
        ..
    } = server_config.clone();
    info!("Connecting to database `{}`", database.debug_message());
    let database = Database::connect(ConnectOptions::new(database.to_string()))
        .await
        .map(Data::new)
        .expect("Failed to connect to database");

    let session_manager = SessionManager::new(session_manager)
        .map(Data::new)
        .expect("Failed to create session manager");

    SessionManager::start_cleaner(session_manager.clone().into_inner());

    let email = EmailService::start(email)
        .await
        .expect("Failed to start email service")
        .map(Data::new)
        .expect("Failed to start email service");

    let password_reset = Data::new(PasswordResetManager {
        email_access: email.clone().into_inner(),
        requests: Default::default(),
    });

    let shared_config = Data::new(SharedConfig {
        password_hash: password_hash_for_new_passwords,
        https: if tls.is_some() { true } else { is_https },
    });

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method()
            .supports_credentials();
        App::new()
            .app_data(database.clone())
            //.app_data(stalwart_manager.clone())
            .app_data(session_manager.clone())
            .app_data(email.clone())
            .app_data(shared_config.clone())
            .app_data(password_reset.clone())
            .wrap(TracingLogger::default())
            .wrap(cors)
            .service(
                Scope::new("/api")
                    .wrap(HandleSession(session_manager.clone()))
                    .configure(api::user::init)
                    .service(Scope::new("/accounts").configure(api::accounts::init))
                    .service(Scope::new("/emails").configure(api::emails::init))
                    .service(Scope::new("/groups").configure(api::groups::init)),
            )
            .service(Scope::new("/frontend-api").configure(frontend::api::init))
    })
    .workers(number_of_workers);
    #[cfg(feature = "rust-tls")]
    return if let Some(TlsConfig {
        private_key,
        certificate_chain,
    }) = tls
    {
        use rustls::{Certificate, PrivateKey, ServerConfig as RustlsServerConfig};
        use rustls_pemfile::{certs, pkcs8_private_keys};
        let mut cert_file = BufReader::new(File::open(certificate_chain)?);
        let mut key_file = BufReader::new(File::open(private_key)?);

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
        server.bind_rustls_021(bind_address, config)?.run().await
    } else {
        server.bind(bind_address)?.run().await
    };
    #[cfg(feature = "native-tls")]
    return if let Some(TlsConfig {
        private_key,
        certificate_chain,
    }) = tls
    {
        use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file(private_key, SslFiletype::PEM)
            .unwrap();
        builder
            .set_certificate_chain_file(certificate_chain)
            .unwrap();
        return server.bind_openssl(bind_address, builder)?.run().await;
    } else {
        return server.bind(bind_address)?.run().await;
    };
}
