use std::fmt::Debug;

use chrono::{DateTime, Utc};

mod scales_s0;

#[derive(Debug)]
struct EventEntry<T: Debug> {
    pub datetime: DateTime<Utc>,
    pub event: T,
}