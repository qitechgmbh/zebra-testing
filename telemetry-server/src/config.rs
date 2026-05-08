use anyhow::anyhow;
use std::{
    env::{VarError, var},
    str::FromStr, sync::Arc, time::Duration,
};

#[derive(Debug)]
pub struct Config {
    pub db_path: String,
    pub socket_path: String,
    pub live_port: u16,
    pub query_port: u16,

    // cache config
    pub channel_capacity: usize,
    pub recorder_cache_size: usize,
    pub live_channel_capacity: usize,

    // timeouts
    pub ingest_read_timeout: Duration,
}

impl Config {
    pub fn init() -> anyhow::Result<Arc<Self>> {
        let db_path 
            = import_as("QITECH_TELEMETRY_DB_PATH")?;

        let socket_path 
            = import_as("QITECH_TELEMETRY_SOCKET_PATH")?;

        let live_port 
            = import_as_or("QITECH_TELEMETRY_LIVE_PORT", 22010)?;

        let query_port 
            = import_as_or("QITECH_TELEMETRY_QUERY_PORT", 22020)?;

        let channel_capacity 
            = import_as_or("QITECH_TELEMETRY_CHANNEL_CAPACITY", 4096)?;

        let recorder_cache_size 
            = import_as_or("QITECH_TELEMETRY_RECORDER_CACHE_SIZE", 32)?;

        let live_channel_capacity 
            = import_as_or("QITECH_TELEMETRY_LIVE_CHANNEL_CAPACITY", 128)?;

        let instance = Self {
            db_path,
            socket_path,
            live_port,
            query_port,
            channel_capacity,
            live_channel_capacity,
            recorder_cache_size,
            ingest_read_timeout: Duration::from_millis(2000),
        };

        Ok(Arc::new(instance))
    }
}

fn import_as<T>(name: &str) -> anyhow::Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let Some(env_var) = import_var(name)? else {
        anyhow::bail!("env var: {name} not found");
    };

    let value = env_var.parse::<T>()?;
    Ok(value)
}

fn import_as_or<T>(name: &str, default: T) -> anyhow::Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match import_var(name)? {
        Some(v) => {
            let value = v.parse::<T>()?;
            Ok(value)
        }
        None => Ok(default),
    }
}

fn import_var(name: &str) -> anyhow::Result<Option<String>> {
    match var(name) {
        Ok(v) => Ok(Some(v)),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(_)) => Err(anyhow!("Env Var {name} not unicode.")),
    }
}
