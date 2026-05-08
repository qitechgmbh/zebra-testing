use std::{fmt, sync::{Arc, atomic::AtomicBool}, time::Instant};

use crate::service::ServiceFactory;

#[derive(Debug)]
pub enum Error {
    FailedToInstallHooks,
    FailedToInitializeService,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::FailedToInstallHooks => {
                write!(f, "failed to install signal hooks")
            }
            Error::FailedToInitializeService => {
                write!(f, "failed to initialize service")
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct ServiceEntry {
    pub name:           String,
    pub factory:        Box<dyn ServiceFactory>,
    pub shutdown_flag:  Arc<AtomicBool>,
    pub last_heartbeat: Instant,
}

impl ServiceEntry {
    pub fn new(name: String, factory: Box<dyn ServiceFactory>,) -> Self {
        Self { 
            name, 
            factory, 
            shutdown_flag:  Default::default(), 
            last_heartbeat: Instant::now(),
        }
    }
}

#[derive(Debug)]
pub enum Signal {
    Shutdown,
    Alive(usize),
    RunnerTerminated(usize),
    RestartFailed(usize, usize),
}