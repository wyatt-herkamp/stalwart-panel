use clap::Parser;
use std::io;
use std::path::PathBuf;

#[derive(Parser)]
struct Command {
    #[clap(short, long)]
    stalwart_config: PathBuf,
}
#[actix_web::main]
async fn main() -> io::Result<()> {
    let _command = Command::parse();

    Ok(())
}
