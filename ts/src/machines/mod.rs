use std::pin::Pin;

use bytes::Bytes;
use tokio::sync::broadcast;

mod scales_s0;

type RecordFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
type RecordFn     = fn(String, usize, broadcast::Receiver<Bytes>) -> RecordFuture;

#[derive(Debug, Clone, Copy)]
pub struct MachineEntry {
    pub tables:     &'static [&'static str],
    pub statements: &'static [&'static str],
    pub validate:   fn(&[u8]) -> bool,
    pub recorder:   RecordFn,
}

