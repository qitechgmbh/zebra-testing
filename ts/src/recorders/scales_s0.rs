use std::sync::Arc;

use duckdb::Connection;
use telemetry_core::{Event, EventKind, LogEvent, OrderEvent, PlateEvent, WeightEvent};

use crate::config::Config;
use super::EventEntry;

// System creates connection to each db initializes and retains connection
// So no other process can lock us out of the db.
// The only way then is if the database is deleted, if this happens -> restart db

#[derive(Debug)]
pub struct ScalesS0Recorder {
    capacity:   usize,
    connection: Connection,
    weights:    Vec<EventEntry<WeightEvent>>,
    plates:     Vec<EventEntry<PlateEvent>>,
    orders:     Vec<EventEntry<OrderEvent>>,
    logs:       Vec<EventEntry<LogEvent>>,
}

impl ScalesS0Recorder {
    pub fn new(config: Arc<Config>) -> anyhow::Result<Self> {
        let capacity   = 128;
        let connection = Connection::open(&config.db_path)?;

        Ok(Self {
            capacity,
            connection,
            weights: Default::default(),
            plates:  Default::default(),
            orders:  Default::default(),
            logs:    Default::default(),
        })
    }

    pub async fn run(mut self) -> anyhow::Result<()> {



        Ok(())
    }
}