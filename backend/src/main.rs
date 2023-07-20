use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use clap::Parser;
use rustls::{Certificate, PrivateKey, ServerConfig as RustlsServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use sea_orm::{ConnectOptions, Database};
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use tokio::fs::read_to_string;
use utils::config::Settings;

#[derive(Parser)]
struct Command {
    #[clap(short, long)]
    config: PathBuf,
}
pub type DatabaseConnection = Data<sea_orm::DatabaseConnection>;
#[actix_web::main]
async fn main() -> io::Result<()> {
    let command = Command::parse();

    let config = read_to_string(command.config).await?;

    let server_config: Settings = toml::from_str(&config).expect("Failed to parse config");

    if !server_config.stalwart_config.exists() {
        // We access the Stalwart Config to add and remove domains
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Stalwart Config not found",
        ));
    }

    let database = Database::connect(ConnectOptions::new(server_config.database.to_string()))
        .await
        .map(Data::new)
        .expect("Failed to connect to database");

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .supports_credentials();
        App::new().wrap(cors).app_data(database.clone())
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
