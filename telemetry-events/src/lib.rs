use chrono::{DateTime, Utc};

mod weight_event;
pub use weight_event::WeightEvent;

mod plate_event;
pub use plate_event::PlateEvent;

mod order_event ;
pub use order_event::OrderEvent;
pub use order_event::WeightBounds;

mod log_event;
pub use log_event::LogEvent;
pub use log_event::LogCategory;

pub type EventSize = u16;
pub const FRAME_SIZE_MAX: usize = 512;

#[derive(Debug, Clone)]
pub struct Event {
    pub datetime: DateTime<Utc>,
    pub kind:     EventKind,
}

impl Event {
    pub fn encode<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        use EventKind::*;

        // timestamp
        buf[0..8].copy_from_slice(&self.datetime.timestamp_micros().to_le_bytes());

        // kind tag
        let kind_id = match &self.kind {
            Weight(_) => 0,
            Plate(_)  => 1,
            Order(_)  => 2,
            Log(_)    => 3,
        };
        buf[8] = kind_id;

        // payload starts at offset 9
        let data_len = match &self.kind {
            Weight(event) => event.encode(&mut buf[9..]).len(),
            Plate(event)  => event.encode(&mut buf[9..]).len(),
            Order(event)  => event.encode(&mut buf[9..]).len(),
            Log(event)    => event.encode(&mut buf[9..]).len(),
        };

        &buf[..9 + data_len]
    }

    pub fn decode(buf: &[u8]) -> Option<Self> {
        use EventKind::*;

        if buf.len() < 9 {
            return None;
        }

        // --- timestamp ---
        let mut ts_bytes = [0u8; 8];
        ts_bytes.copy_from_slice(&buf[0..8]);
        let ts = i64::from_le_bytes(ts_bytes);

        let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp_micros(ts)?;

        // --- kind ---
        let kind_id = buf[8];
        let payload = &buf[9..];

        let kind = match kind_id {
            0 => {
                let event = WeightEvent::decode(payload)?;
                Weight(event)
            }
            1 => {
                let event = PlateEvent::decode(payload)?;
                Plate(event)
            }
            2 => {
                let event = OrderEvent::decode(payload)?;
                Order(event)
            }
            3 => {
                let event = LogEvent::decode(payload)?;
                Log(event)
            }
            _ => return None, // unknown kind
        };

        Some(Event {
            datetime,
            kind,
        })
    }
}

#[derive(Debug, Clone)]
pub enum EventKind {
    Weight(WeightEvent),
    Plate(PlateEvent),
    Order(OrderEvent),
    Log(LogEvent),
}

// timestamp
// let timestamp = self.datetime.timestamp_micros().to_le_bytes();
// buf[i..i + 8].copy_from_slice(&timestamp);
// i += 8;

// let timestamp = i64::from_le_bytes(buf[i..i + 8].try_into().unwrap());
// let datetime  = DateTime::from_timestamp_nanos(timestamp);
