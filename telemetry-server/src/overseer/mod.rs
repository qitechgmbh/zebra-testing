use std::{mem, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};

use telemetry_core::{Event, FRAME_SIZE_MAX};
use tokio::{io::{self, AsyncReadExt}, net::{TcpStream, UnixListener, UnixStream}, sync::mpsc::{Receiver, Sender}, task::{JoinHandle, JoinSet}, time::{sleep, timeout}};

pub type ServiceId = u32;
pub type RunnerId  = u64;

mod ingest;
mod task_runner;


pub enum EventSource {
    FF01,
    FF02,
}

pub enum Signal {
    Shutdown, 



    SpawnIngest(EventSource, UnixStream),
    SpawnQuery(EventSource, TcpStream),
    SpawnRecorder(String),

    RestartIngest(io::Error),
    RestartRecorder,
    RestartLive,
    RestartQuery,
    IngestStartProcessing(UnixStream),
    DistributeEvent(Arc<Vec<u8>>),
}

// service died: with ID, overseer has the service dyn type to create it again

pub struct Overseer {
    rx: Receiver<Signal>,
    tx: Sender<Signal>,

    ingest: IngestService,
}

impl Overseer {
    pub async fn run(mut self) {
        // start all launchers

        self.ingest.state = IngestServiceState::Listen(

        );

        loop {
            let signal = self.rx.recv().await.expect("rx and tx share lifetime");

            match signal {
                Signal::Shutdown => {
                    break;
                }

                Signal::RestartIngest => {

                }

                Signal::RestartRecorder => {

                }

                Signal::RestartLive => {

                }

                Signal::RestartQuery => {

                }

                Signal::IngestStartProcessing(stream) => {


                    self.ingest.state = 
                        IngestServiceState::Process();
                }

                Signal::DistributeEvent(items) => {

                }
            }
        }   

        // shut down ingest service first
        match self.ingest.state {
            IngestServiceState::Offline => {}

            IngestServiceState::Listen(task)
            | IngestServiceState::Process(task) => {
                task.shutdown().await;
            }
        }
    }
}

enum ShutdownPolicy {
    Immediate,
    Graceful(Duration),
}

struct Task {
    flag:   Arc<AtomicBool>,
    timer:  Duration,
    handle: JoinHandle<()>,
}

impl Task {
    async fn shutdown(mut self) {
        self.flag.store(true, Ordering::Release);

        match timeout(self.timer, &mut self.handle).await {
            Ok(join_res) => {
                let _ = join_res;
            }

            Err(_) => {
                self.handle.abort();
            }
        }
    }
}

pub async fn ingest_listen(
    listener: Arc<UnixListener>, 
    tx: Sender<Signal>
) {
    let (stream, _) = match listener.accept().await {
        Ok(v) => v,
        Err(e) => {
            tx.send(Signal::RestartIngest(e)).await
                .expect("tx is tied to overseer lifetime");

            return;
        }
    };

    tx.send(Signal::IngestStartProcessing(stream)).await
        .expect("tx is tied to overseer lifetime");

}


pub struct IngestService {
    pub listener: Arc<UnixListener>,
    pub state:    IngestServiceState,
}

#[derive(Default)]
enum IngestServiceState {
    #[default]
    Offline,
    Listen(Task<()>),
    Process(Task<()>),
    Aborted(io::Error)
}

pub struct IngestListener {
    
}

pub struct IngestRunner {
    stream: UnixStream
}

impl IngestRunner {
    pub async fn run(&mut self) -> io::Result<RunnerStatus> {
        let mut buf = [0u8; FRAME_SIZE_MAX];

        let len = self.stream.read_u32().await? as usize;
        self.stream.read_exact(&mut buf[0..len]).await?;

        let data = &buf[..len];

        // if data is malformed we are out of sync with the stream
        if Event::decode(data).is_none() {
            return Ok(RunnerStatus::Finished);

            // self.errors.push(ServiceError::new(
            //     ServiceErrorSeverity::Low,
            //     "Received malformed data. Discarding connection!", 
            // ));
            // return Ok(State::FindConnection);
        };

        Ok(RunnerStatus::Running)
    }
}


pub struct QueryServiceConfig {
    port: u16
}

pub struct QueryService {
    pub receiver: Receiver<QueryServiceMail>,
    pub mailbox:  Vec<QueryServiceMail>,
    pub listener: JoinHandle<()>,
    pub clients:  Vec<Task>
}

pub struct Session {
    pub receiver: Receiver<QueryServiceMail>,
    pub mailbox:  Vec<QueryServiceMail>,

    pub listener: JoinHandle<()>,
    pub clients:  Vec<Task>
}

impl QueryService {
    pub async fn new(tx: Sender<Signal>, port: u16) {
        
    }

    pub async fn shutdown(&mut self) {
        self.listener.abort();

        let mut set = JoinSet::new();

        for client in mem::take(&mut self.clients) {
            set.spawn(async move {
                client.shutdown().await;
            });
        }

        while let Some(res) = set.join_next().await {
            if let Err(e) = res {
                eprint!("[Query] Error while shutting down client: {e}");
            }
        }
    }

    pub async fn process_mailbox(&mut self) {

    }
}

pub enum QueryServiceMail {
    ClientAdded()
}