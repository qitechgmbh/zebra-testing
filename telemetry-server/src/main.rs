use std::sync::Arc;
use crossbeam::channel::{Receiver, Sender, bounded};

// use overseer::{Overseer, ServiceFactory};

mod overseer;
mod actors;

mod config;
use config::Config;

mod utils;
mod system;

mod services;
use services::IngestService;
use services::IngestServiceFactory;

type Payload = Arc<Vec<u8>>;
type PayloadSender = Sender<Payload>;
type PayloadReceiver = Receiver<Payload>;

fn main() -> anyhow::Result<()> {
    let config = Config::init()?;
    println!("Loaded Config: {:?}", &config);

    utils::init_db(&config.db_path)?;

    let (tx_recorder, rx_recorder) = bounded::<Payload>(config.channel_capacity);
    let (tx_live, rx_live)         = bounded::<Payload>(config.channel_capacity);

    let rx_recorder = Arc::new(rx_recorder);
    let rx_live     = Arc::new(rx_live);

    let services: Vec<(String, Box<dyn ServiceFactory>)> = vec![
        ("Ingest".to_string() ,Box::new(IngestServiceFactory { 
            config,
             subscribers: vec![tx_recorder, tx_live] 
        }))
    ];

    let overseer = Overseer::new(services)?;
    overseer.run()?;

    Ok(())
}