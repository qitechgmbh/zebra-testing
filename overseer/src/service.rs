use std::fmt::Debug;

use chrono::{DateTime, Utc};

pub trait ServiceFactory where Self: Debug {
    fn new(&self) -> anyhow::Result<Box<dyn Service>>;
}

pub trait Service where Self: Send {
    fn cycle(&mut self) -> CycleStatus;
    fn shutdown(&mut self) -> CycleStatus;
    fn take_errors(&mut self) -> Vec<ServiceError>;
}

#[derive(Debug)]
pub enum CycleStatus {
    Healthy,
    Abort(anyhow::Error),
}

#[derive(Debug)]
pub struct ServiceError {
    pub timestamp: DateTime<Utc>,
    pub message:   &'static str,
    pub severity:  ServiceErrorSeverity,
}

impl ServiceError {
    pub fn new(severity: ServiceErrorSeverity, message: &'static str) -> Self {
        Self { timestamp: Utc::now(), message, severity }
    }
}

#[derive(Debug)]
pub enum ServiceErrorSeverity {
    Low,
    Medium,
    High,
}