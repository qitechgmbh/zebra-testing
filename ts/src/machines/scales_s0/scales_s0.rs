use bytes::Bytes;
use tokio::sync::broadcast;

use super::MachineDataSchema;

pub const SCALES_S0: MachineDataSchema = MachineDataSchema {
    tables: &[
        "weights",
        "plates",
        "orders",
        "logs",
    ],
    statements: &[
        "CREATE TYPE IF NOT EXISTS OrderStatus AS ENUM ('Started', 'Aborted', 'Completed');",

        "CREATE TYPE IF NOT EXISTS LogCategory AS ENUM ('Debug', 'Info', 'Warn', 'Error');", 

        "CREATE TYPE IF NOT EXISTS WeightBounds AS STRUCT (
            min     SMALLINT, 
            max     SMALLINT, 
            desired SMALLINT, 
            trigger SMALLINT
        );", 

        "CREATE TABLE IF NOT EXISTS weights (
            timestamp TIMESTAMP_NS,
            order_id  UINTEGER,
            weight_0  SMALLINT,
            weight_1  SMALLINT
        )",

        "CREATE TABLE IF NOT EXISTS plates (
            timestamp TIMESTAMP_NS,
            order_id  UINTEGER,
            peak SMALLINT NOT NULL,
            real SMALLINT NOT NULL
        )",

        "CREATE TABLE IF NOT EXISTS orders (
            order_id       UINTEGER PRIMARY KEY,
            worker_id      UINTEGER,
            status         OrderStatus NOT NULL,
            bounds         WeightBounds,
            quantity_good  UINTEGER,
            quantity_scrap UINTEGER,
            started_at     TIMESTAMP_NS NOT NULL,
            closed_at      TIMESTAMP_NS
        )",

        "CREATE TABLE IF NOT EXISTS logs (
            timestamp TIMESTAMP_NS NOT NULL,
            category  LogCategory NOT NULL,
            message   VARCHAR NOT NULL
        )",
    ],

    validate: validate_scales_s0,
    recorder: record,
};

fn validate_scales_s0(data: &[u8]) -> bool {
    telemetry_core::Event::decode(data).is_some()
}


// fn(String, usize, broadcast::Receiver<Bytes>) -> RecordFuture;
async fn record(db_path: String, capacity: usize, rx: broadcast::Receiver<Bytes>) {

}