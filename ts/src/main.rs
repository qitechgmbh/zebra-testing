mod config;
use config::Config;

mod overseer;
mod types;
mod system;
mod router;
mod ingest;
mod recorder;
mod recorders;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::default();
    println!("Loaded Config: {:?}", &config);

    system::run(config).await?;

    Ok(())
}

// recorders: task that receives vecu8 just box it??