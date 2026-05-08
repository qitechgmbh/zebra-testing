use std::path::Path;
use duckdb::Connection;

pub fn init_db<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    let connection = Connection::open(path)?;

    connection.execute(
        "CREATE TYPE IF NOT EXISTS OrderStatus AS ENUM ('Started', 'Aborted', 'Completed');", 
        []
    )?;

    connection.execute(
        "CREATE TYPE IF NOT EXISTS LogCategory AS ENUM ('Debug', 'Info', 'Warn', 'Error');", 
        []
    )?;

    connection.execute(
        "CREATE TYPE IF NOT EXISTS WeightBounds AS STRUCT (
            min     SMALLINT, 
            max     SMALLINT, 
            desired SMALLINT, 
            trigger SMALLINT
        );", 
        []
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS weights (
            timestamp TIMESTAMP_NS,
            order_id  UINTEGER,
            weight_0  SMALLINT,
            weight_1  SMALLINT
        )",
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS plates (
            timestamp TIMESTAMP_NS,
            order_id  UINTEGER,
            peak SMALLINT NOT NULL,
            real SMALLINT NOT NULL
        )",
        [],
    )?;

    connection.execute(
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
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS logs (
            timestamp TIMESTAMP_NS NOT NULL,
            category  LogCategory NOT NULL,
            message   VARCHAR NOT NULL
        )",
        [],
    )?;

    Ok(())
}