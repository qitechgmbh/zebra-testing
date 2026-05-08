use std::{panic, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Instant};
use crossbeam::channel::Sender;

use crate::types::Signal;
use crate::service::{CycleStatus, Service};

pub struct ServiceRunner {
    // communication
    pub flag:    Arc<AtomicBool>,
    pub supervisor_tx: Sender<Signal>,

    // state
    pub name:    String,
    pub service: Box<dyn Service>,
}

impl ServiceRunner {
    pub fn run(mut self, id: usize) {
        let flag = self.flag.clone();

        let mut last_alive = Instant::now();
        let run_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            while !flag.load(Ordering::Acquire) {
                if let CycleStatus::Abort(e) = self.service.cycle() {
                    eprintln!("[{}] Fatal Error while running: {}", self.name, e);
                    break;
                }

                let now = Instant::now();
                if now.duration_since(last_alive).as_secs_f64() > 2.5 {
                    _ = self.supervisor_tx.send(Signal::Alive(id));
                    last_alive = now;
                }
            }
        }));

        if let Err(panic) = run_result {
            eprintln!(
                "[{}] Panicked while running: {:?}", 
                self.name, 
                panic_message(panic),
            );
            eprintln!("[{}] Attempting to shut down service", self.name);
        }

        let shutdown_result = panic::catch_unwind(
            panic::AssertUnwindSafe(|| self.service.shutdown())
        );

        if let Err(panic) = shutdown_result {
            eprintln!(
                "[{}] Panicked while shutting down: {:?}", 
                self.name, 
                panic_message(panic)
            );
        }

        // notify supervisor that service terminated
        _ = self.supervisor_tx.send(Signal::RunnerTerminated(id));
    }
}

fn panic_message(panic: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown Panic payload".into()
    }
}