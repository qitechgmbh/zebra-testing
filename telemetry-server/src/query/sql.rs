use std::collections::HashMap;
use chrono::{DateTime, NaiveDateTime};
use duckdb::types::{TimeUnit, Value};
use crate::query::args::QueryArgs;

#[derive(Debug, Clone)]
pub enum FieldType {
    I16,
    U32,
    Message,
    Timestamp,
    WeightBounds,
    LogCategory,
    OrderStatus,
}

pub fn create(
    table:  &str,
    fields: HashMap<String, FieldType>,
    query:  QueryArgs,
) -> anyhow::Result<(String, Vec<Value>)> {
    let mut sql = String::new();

    // SELECT ?
    let select = parse_select(&fields, &query)?;
    sql.push_str(&select);

    // FROM ?
    sql.push_str(" FROM ");
    sql.push_str(table);
    sql.push_str(" ");

    // WHERE ?
    let (r#where, params) = parse_where(&fields, &query)?;
    sql.push_str(&r#where);

    // ORDER BY
    let order_by = parse_order_by(&fields, &query)?;
    sql.push_str(&order_by);

    Ok((sql, params))
}

fn parse_select(
    fields: &HashMap<String, FieldType>,
    query:  &QueryArgs
) -> anyhow::Result<String> {
    let mut out = String::new();

    let select = query.get_csv("select")?;

    if !select.is_empty() {
        out.push_str(" SELECT ");

        for (i, raw) in select.iter().enumerate() {
            let field = raw.trim();

            if !fields.contains_key(field) {
                anyhow::bail!("Illegal item in select: {field}");
            };

            if i > 0 {
                out.push_str(", ");
            }

            out.push_str(field);
        }

        Ok(out)
    } else {
        Ok(" SELECT * ".to_string())
    }
}

fn parse_where(
    fields: &HashMap<String, FieldType>,
    query: &QueryArgs,
) -> anyhow::Result<(String, Vec<Value>)> {
    let mut sql    = String::new();
    let mut params = Vec::new();

    let conditions = query.get_csv("where")?;

    if conditions.is_empty() {
        return Ok((sql, params));
    }

    sql.push_str(" WHERE ");

    for (i, cond) in conditions.iter().enumerate() {
        let cond = cond.trim();

        if cond.is_empty() {
            anyhow::bail!("Empty WHERE condition");
        }

        const OPS: &[&str] = &["!=", ">=", "<=", "==", ">", "<"];

        let mut found = None;

        for op in OPS {
            if let Some(pos) = cond.find(op) {
                found = Some((pos, *op));
                break;
            }
        }

        let (pos, op) = found
            .ok_or_else(|| anyhow::anyhow!("Missing operator in WHERE: {cond}"))?;

        let field = cond[..pos].trim();
        let value = cond[pos + op.len()..].trim();

        let Some(field_type) = fields.get(field) else {
            anyhow::bail!("Unknown field in WHERE: {field}");
        };

        let parsed_value = match field_type.to_owned() {
            FieldType::U32 => {
                Value::UInt(value.parse()?)
            }
            FieldType::I16 => {
                Value::Int(value.parse()?)
            }
            FieldType::Message => {
                let v = value.trim_matches('"').to_string();
                Value::Text(v)
            }
            FieldType::Timestamp => {
                parse_datetime(value)?
            }
            FieldType::WeightBounds => {
                anyhow::bail!("WeightBounds not supported inside $where")
            }
            FieldType::LogCategory => {
                match value.trim().to_lowercase().as_str() {
                    "debug" => Value::Enum("Debug".to_string()),
                    "info"  => Value::Enum("Info".to_string()),
                    "warn"  => Value::Enum("Warn".to_string()),
                    "error" => Value::Enum("Error".to_string()),
                    _ => anyhow::bail!("Invalid LogCategory"),
                }
            }
            FieldType::OrderStatus => {
                match value.trim().to_lowercase().as_str() {
                    "started"   => Value::Enum("Started".to_string()),
                    "completed" => Value::Enum("Completed".to_string()),
                    "aborted"   => Value::Enum("Aborted".to_string()),
                    _ => anyhow::bail!("Invalid OrderStatus"),
                }
            }
        };

        if i > 0 {
            sql.push_str(" AND ");
        }

        sql.push_str(field);
        sql.push(' ');
        sql.push_str(op);
        sql.push_str(" ?");

        params.push(parsed_value);
    }

    Ok((sql, params))
}

fn parse_order_by(
    fields: &HashMap<String, FieldType>,
    query:  &QueryArgs
) -> anyhow::Result<String> {
    let mut out = String::new();

    let order_by = query.get_csv("order_by")?;

    if !order_by.is_empty() {
        out.push_str(" ORDER BY ");

        for (i, raw) in order_by.iter().enumerate() {
            let mut parts = raw.split_whitespace();

            let field = parts.next()
                .ok_or_else(|| anyhow::anyhow!("Empty order_by field"))?;

            let direction = match parts.next() {
                Some("ASC") | None => "ASC",
                Some("DESC") => "DESC",
                Some(other) => anyhow::bail!("Invalid sort direction: {other}"),
            };

            if parts.next().is_some() {
                anyhow::bail!("Too many tokens in order_by: {raw}");
            }

            let Some(_field_type) = fields.get(field) else {
                anyhow::bail!("Illegal item in order_by: {field}");
            };

            if i > 0 {
                out.push_str(", ");
            }

            out.push_str(field);
            out.push(' ');
            out.push_str(direction);
        }
    }

    Ok(out)
}

fn parse_datetime(input: &str) -> anyhow::Result<Value> {
    // timezone-aware formats
    const TZ_FORMATS: &[&str] = &[
        "%Y-%m-%d %H:%M:%S%.f%:z",
        "%Y-%m-%d %H:%M:%S%.f%z",
    ];

    for fmt in TZ_FORMATS {
        if let Ok(dt) = DateTime::parse_from_str(input, fmt) {
            return Ok(Value::Timestamp(
                TimeUnit::Microsecond,
                dt.timestamp_micros(),
            ));
        }
    }

    // naive UTC fallback
    const NAIVE_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.f";

    if let Ok(dt) = NaiveDateTime::parse_from_str(input, NAIVE_FORMAT) {
        return Ok(Value::Timestamp(
            TimeUnit::Microsecond,
            dt.and_utc().timestamp_micros(),
        ));
    }

    anyhow::bail!("Invalid datetime format")
}