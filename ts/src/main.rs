use std::io;

mod config;
use config::Config;

mod types;
mod stream;

mod machines;
mod producer;
mod consumer;

mod system;

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = Config::testing();
    println!("Loaded Config");

    println!("Starting System...");
    system::run(config).await?;

    Ok(())
}