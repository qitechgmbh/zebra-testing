use chrono::{DateTime, Utc};
use duckdb::{Connection, params};
use std::{fmt::Debug, sync::Arc};

use telemetry_core::{Event, EventKind, LogEvent, OrderEvent, PlateEvent, WeightEvent};

use crate::{PayloadReceiver, config::Config};

#[derive(Debug)]
struct EventEntry<T: Debug> {
    pub datetime: DateTime<Utc>,
    pub event: T,
}
#[derive(Debug)]
pub struct Recorder {
    capacity:   usize,
    connection: Connection,
    weights:    Vec<EventEntry<WeightEvent>>,
    plates:     Vec<EventEntry<PlateEvent>>,
    orders:     Vec<EventEntry<OrderEvent>>,
    logs:       Vec<EventEntry<LogEvent>>,
}

impl Recorder {
    pub fn new(config: Arc<Config>) -> anyhow::Result<Self> {
        let capacity   = config.recorder_cache_size;
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

    pub fn run(mut self, rx: Arc<PayloadReceiver>) -> anyhow::Result<()> {
        loop {
            let data = match rx.recv() {
                Ok(v) => v,
                Err(e) => {
                    // try to flush all before exiting
                    _ = self.flush_weights();
                    _ = self.flush_plates();
                    _ = self.flush_orders();
                    _ = self.flush_logs();
                    return Err(e.into());
                }
            };

            let event = Event::decode(&data).expect("Ingest must validate data before sending");
            self.append_entry(event)?;
        }
    }

    fn append_entry(&mut self, event: Event) -> anyhow::Result<()> {
        match event.kind {
            EventKind::Weight(kind) => {
                let entry = EventEntry {
                    datetime: event.datetime,
                    event:    kind,
                };

                if self.weights.len() >= self.capacity {
                    self.flush_weights()?;
                }

                self.weights.push(entry);
            }
            EventKind::Plate(kind) => {
                let entry = EventEntry {
                    datetime: event.datetime,
                    event:    kind,
                };

                if self.plates.len() >= self.capacity {
                    self.flush_plates()?;
                }

                self.plates.push(entry);
            }
            EventKind::Order(kind) => {
                let entry = EventEntry {
                    datetime: event.datetime,
                    event:    kind,
                };

                if self.orders.len() >= self.capacity {
                    self.flush_orders()?;
                }

                self.orders.push(entry);
            }
            EventKind::Log(kind) => {
                let entry = EventEntry {
                    datetime: event.datetime,
                    event: kind,
                };

                if self.logs.len() >= self.capacity {
                    self.flush_orders()?;
                }

                self.logs.push(entry);
            }
        }

        Ok(())
    }

    fn flush_weights(&mut self) -> anyhow::Result<()> {
        if self.weights.is_empty() {
            return Ok(());
        }

        let tx = self.connection.transaction()?;
        let mut stmt = tx.prepare("INSERT INTO weights VALUES (?, ?, ?, ?)")?;

        for entry in self.weights.drain(..) {
            let datetime = datetime_to_str(entry.datetime);

            stmt.execute(params![
                datetime,
                entry.event.order_id,
                entry.event.weight_0,
                entry.event.weight_1
            ])?;
        }

        tx.commit()?;

        Ok(())
    }

    fn flush_plates(&mut self) -> anyhow::Result<()> {
        if self.plates.is_empty() {
            return Ok(());
        }

        let tx = self.connection.transaction()?;
        let mut stmt = tx.prepare("INSERT INTO plates VALUES (?, ?, ?, ?)")?;

        for entry in self.plates.drain(..) {
            let datetime = datetime_to_str(entry.datetime);

            stmt.execute(params![
                datetime,
                entry.event.order_id,
                entry.event.peak,
                entry.event.real,
            ])?;
        }
        tx.commit()?;

        Ok(())
    }

    fn flush_orders(&mut self) -> anyhow::Result<()> {
        if self.orders.is_empty() {
            return Ok(());
        }

        let tx = self.connection.transaction()?;

        let mut stmt_started = tx.prepare(
            r#"
            INSERT INTO orders 
            VALUES (?, ?, ?, row(?, ?, ?, ?), ?, ?, ?, ?)
        "#,
        )?;

        let mut stmt_aborted = tx.prepare(
            r#"
            UPDATE orders
            SET status = 'Aborted',
                closed_at = ?
            WHERE order_id = ?
        "#,
        )?;

        let mut stmt_completed = tx.prepare(
            r#"
            UPDATE orders
            SET status = 'Completed',
                quantity_good = ?,
                quantity_scrap = ?,
                closed_at = ?
            WHERE order_id = ?
        "#,
        )?;

        for entry in self.orders.drain(..) {
            let datetime = datetime_to_str(entry.datetime);

            match entry.event {
                OrderEvent::Started {
                    order_id,
                    worker_id,
                    bounds,
                } => {
                    stmt_started.execute(params![
                        order_id,
                        worker_id,
                        "Started",
                        bounds.as_ref().map(|b| b.min),
                        bounds.as_ref().map(|b| b.max),
                        bounds.as_ref().map(|b| b.desired),
                        bounds.as_ref().map(|b| b.trigger),
                        0,
                        0,
                        datetime,
                        None::<String>
                    ])?;
                }

                OrderEvent::Completed {
                    order_id,
                    quantity_good,
                    quantity_scrap,
                } => {
                    stmt_completed.execute(params![
                        quantity_good,
                        quantity_scrap,
                        datetime,
                        order_id,
                    ])?;
                }

                OrderEvent::Aborted { order_id } => {
                    stmt_aborted.execute(params![datetime, order_id,])?;
                }
            }
        }

        tx.commit()?;
        Ok(())
    }

    fn flush_logs(&mut self) -> anyhow::Result<()> {
        if self.logs.is_empty() {
            return Ok(());
        }

        let tx = self.connection.transaction()?;
        let mut stmt = tx.prepare("INSERT INTO logs VALUES (?, ?, ?)")?;

        for entry in self.logs.drain(..) {
            let datetime = datetime_to_str(entry.datetime);

            stmt.execute(params![
                datetime,
                entry.event.category.to_str(),
                entry.event.message.as_bytes(),
            ])?;
        }
        tx.commit()?;

        Ok(())
    }
}

fn datetime_to_str(datetime: DateTime<Utc>) -> String {
    format!("{}", datetime.format("%Y-%m-%d %H:%M:%S%.f"))
}