use std::{collections::HashMap, net::TcpStream};

use arrow::ipc::writer::StreamWriter;
use duckdb::{Statement, params_from_iter, types::Value};

use crate::query::{http, sql::FieldType};

pub fn destruct_path<'a>(path: &'a str) -> (&'a str, Option<&'a str>)  {
    match path.split_once('?') {
        Some((r, q)) => (r, Some(q)),
        None => (path, None),
    }
}

pub fn respond_arrow(
    mut statement: Statement<'_>, 
    stream: &mut TcpStream, 
    params: Vec<Value>
) -> anyhow::Result<()> {
    let iterator = statement.query_arrow(
        params_from_iter(params)
    )?;

    http::start_arrow_stream(stream)?;

    for batch in iterator {
        let mut buffer = Vec::new();
        {
            let schema = &batch.schema();
            let mut writer = StreamWriter::try_new(&mut buffer, schema)?;
            writer.write(&batch)?;
            writer.finish()?;
        }

        http::write_arrow_batch(stream, &buffer)?;
    }

    http::finish_arrow_stream(stream)?;

    Ok(())
}

pub fn weights_fields() -> HashMap<String, FieldType> {
    let mut fields = HashMap::new();
    fields.insert("timestamp".into(), FieldType::Timestamp);
    fields.insert("order_id".into(),  FieldType::U32);
    fields.insert("weight_0".into(),  FieldType::I16);
    fields.insert("weight_1".into(),  FieldType::I16);
    fields
}

pub fn plates_fields() -> HashMap<String, FieldType> {
    let mut fields = HashMap::<String, FieldType>::new();
    fields.insert("timestamp".into(),  FieldType::Timestamp);
    fields.insert("order_id".into(),   FieldType::U32);
    fields.insert("peak".into(),       FieldType::I16);
    fields
}

pub fn orders_fields() -> HashMap<String, FieldType> {
    let mut fields = HashMap::<String,     FieldType>::new();
    fields.insert("order_id".into(),       FieldType::U32);
    fields.insert("worker_id".into(),      FieldType::U32);
    fields.insert("status".into(),         FieldType::OrderStatus);
    fields.insert("bounds".into(),         FieldType::WeightBounds);
    fields.insert("quantity_good".into(),  FieldType::U32);
    fields.insert("quantity_scrap".into(), FieldType::U32);
    fields.insert("started_at".into(),     FieldType::Timestamp);
    fields.insert("closed_at".into(),      FieldType::Timestamp);
    fields
}

pub fn logs_fields() -> HashMap<String, FieldType> {
    let mut fields = HashMap::<String, FieldType>::new();
    fields.insert("timestamp".into(),  FieldType::Timestamp);
    fields.insert("category".into(),   FieldType::LogCategory);
    fields.insert("message".into(),    FieldType::Message);
    fields
}