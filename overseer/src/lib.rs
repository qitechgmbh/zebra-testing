use std::{sync::atomic::Ordering, thread, time::{Duration, Instant}};

use crossbeam::channel::{Receiver, RecvTimeoutError, Sender, unbounded};
use signal_hook::{consts::{SIGINT, SIGTERM}, iterator::Signals};

mod types;
pub use types::Error;
use types::Signal;
use types::ServiceEntry;

mod service;
pub use service::Service;
pub use service::CycleStatus;
pub use service::ServiceFactory;
pub use service::ServiceError;
pub use service::ServiceErrorSeverity;

mod runner;
use runner::ServiceRunner;

#[derive(Debug)]
pub struct Overseer {
    entries:    Vec<ServiceEntry>,
    sender:     Sender<Signal>,
    receiver:   Receiver<Signal>,
    last_check: Instant,
}

impl Overseer {
    pub fn new(
        services: Vec<(String, Box<dyn ServiceFactory>)>
    ) -> Result<Self, Error> {
        let (sender, receiver) = unbounded();

        install_signal_handler(sender.clone())?;

        let mut entries = Vec::new();

        for (name, factory) in services {
            entries.push(ServiceEntry::new(name, factory));
        }

        Ok(Self {
            entries,
            sender,
            receiver,
            last_check: Instant::now(),
        })
    }

    pub fn run(mut self) -> Result<(), Error> {
        self.try_start_all()?;

        loop {
            match self.receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(signal) => {
                    let now = Instant::now();
                    let shutdown = self.handle_signal(signal, now);
                    if shutdown { break; }
                }

                Err(RecvTimeoutError::Timeout) => {
                    let now = Instant::now();
                    self.check_alive(now);
                }

                Err(RecvTimeoutError::Disconnected) => {
                    unreachable!("Shares lifetime with Sender");
                }
            };
        }

        self.shutdown();
        Ok(())
    }

    fn handle_signal(&mut self, signal: Signal, now: Instant) -> bool {
        match signal {
            Signal::Alive(id) => {
                self.entries[id].last_heartbeat = now;
            }

            Signal::RunnerTerminated(id) => {
                let entry = &self.entries[id];
                entry.shutdown_flag.store(true, Ordering::Release);
                try_restart(self.sender.clone(), entry, id, 0);
            }

            Signal::RestartFailed(id, attempts) => {
                let entry = &self.entries[id];
                try_restart(self.sender.clone(), entry, id, attempts);
            }

            Signal::Shutdown => {
                // request shutdown
                return true;
            }
        }

        false
    }

    fn check_alive(&mut self, now: Instant) {
        if now.duration_since(self.last_check).as_secs_f32() <= 10.0 {
            return;
        }

        for entry in &self.entries {
            let passed = now.duration_since(entry.last_heartbeat).as_secs_f64();

            if passed > 10.0 {
                eprintln!(
                    "Service {} hasn't sent heartbeat for {}s",
                    entry.name,
                    passed
                );
            }
        }

        self.last_check = now;
    }

    fn try_start_all(&mut self) -> Result<(), Error> {
        println!("Starting up services...");

        let mut attempts = 0;

        while attempts < 10 {
            let mut runners  = Vec::new();

            // startup phase
            for entry in &self.entries {
                println!("Initializing service \"{}\"", entry.name);

                let service = match entry.factory.new() {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        attempts += 1;
                        continue;
                    },
                };

                println!("Ok");

                let runner = ServiceRunner {
                    flag:          entry.shutdown_flag.clone(),
                    supervisor_tx: self.sender.clone(),
                    name:          entry.name.clone(),
                    service,
                };

                runners.push(runner);
            }

            let mut id: usize = 0;
            for runner in runners {
                launch_runner(runner, id);
                id += 1;
            }

            return Ok(());
        }

        Err(Error::FailedToInitializeService)
    }

    fn shutdown(self) {
        println!("Shutting Down...");

        for (i, entry) in self.entries.iter().enumerate() {
            println!("Shutting Down \"{}\"...", entry.name);

            if entry.shutdown_flag.swap(true, Ordering::Release) {
                continue;
            }

            let start = Instant::now();

            loop {
                let signal = match self.receiver.recv_timeout(Duration::from_secs(1)) {
                    Ok(s) => s,
                    Err(RecvTimeoutError::Timeout) => {
                        if start.elapsed().as_secs() > 10 {
                            eprintln!("Error: Failed to shutdown in time!");
                            break;
                        }

                        continue;
                    }
                    Err(RecvTimeoutError::Disconnected) => unreachable!(),
                };

                if let Signal::RunnerTerminated(id) = signal {
                    if id == i {
                        println!("Ok");
                        break;
                    }
                }
            }
        }

        println!("Shutdown Complete");
    }
}

fn install_signal_handler(tx: Sender<Signal>) -> Result<(), Error> {
    let mut signals = match Signals::new([SIGTERM, SIGINT]) {
        Ok(v) => v,
        Err(_) => return Err(Error::FailedToInstallHooks),
    };

    thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGTERM | SIGINT => {
                    _ = tx.send(Signal::Shutdown);
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(())
}

fn try_restart(tx: Sender<Signal>, entry: &ServiceEntry, id: usize, attempts: usize) {
    let service = match entry.factory.new() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to initialize service {}. Reason: {}", entry.name, e);

            thread::spawn(move || {
                // ramp up timeout up until limit of 10 seconds
                let timeout = (100 * attempts as u64).min(10000);
                thread::sleep(Duration::from_millis(timeout));

                tx.send(Signal::RestartFailed(id, attempts + 1))
                    .expect("Receiver lives as long as Sender");
            });

            return;
        },
    };

    let runner = ServiceRunner {
        name:          entry.name.clone(),
        flag:          entry.shutdown_flag.clone(),
        supervisor_tx: tx.clone(),
        service,
    };

    entry.shutdown_flag.store(false, Ordering::Release);
    launch_runner(runner, id);
}

fn launch_runner(runner: ServiceRunner, id: usize) {
    println!("Launching service: \"{}\" with id {}", runner.name, id);
    thread::spawn(move || runner.run(id));
    println!("Ok");
}