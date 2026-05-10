use std::{
    env::{VarError, var},
    str::FromStr, sync::Arc,
};
use anyhow::anyhow;
use telemetry_core::MachineDataSchemas;

#[derive(Debug)]
pub struct Config {
    pub db_path: String,
    pub sock_path: String,

    pub tcp_ipv4: String,
    pub tcp_port: u16,

    pub machines: Vec<(String, MachineDataSchemas)>,
}

impl Config {
    pub fn default() -> Arc<Self> {
        let instance = Self {
            db_path:   "idk".into(),
            sock_path: "/tmp/qitech-telemetry.sock".into(),
            tcp_port:  9000,
            tcp_ipv4:  "0.0.0.0".into(),
            machines:  vec![("scales_ff01".into(), MachineDataSchemas::ScalesS0)]
        };

        Arc::new(instance)
    }

    /*
    pub fn init() -> anyhow::Result<Arc<Self>> {
        let db_path 
            = import_as("QITECH_TELEMETRY_DB_PATH")?;

        let sock_path 
            = import_as("QITECH_TELEMETRY_SOCKET_PATH")?;

        let tcp_port 
            = import_as_or("QITECH_TELEMETRY_LIVE_PORT", 22010)?;

        let instance = Self {
            db_path,
            sock_path,
            tcp_port,
            tcp_ipv4: "0.0.0.0".into(),
        };

        Ok(Arc::new(instance))
    }
    */
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
