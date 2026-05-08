use crossbeam::channel::TrySendError;
use overseer::{CycleStatus, Service, ServiceError, ServiceErrorSeverity, ServiceFactory};
use std::{
    fs, io::{self, ErrorKind, Read}, mem, os::unix::net::{UnixListener, UnixStream}, sync::Arc, thread, time::Duration
};

use telemetry_core::{Event, EventSize, FRAME_SIZE_MAX};
use crate::{PayloadSender, config::Config};

#[derive(Debug, Default)]
enum State {
    #[default]
    FindConnection,
    ReadLength(UnixStream),
    ReadBody(UnixStream, usize),
}

#[derive(Debug)]
pub struct IngestServiceFactory {
    pub config:      Arc<Config>,
    pub subscribers: Vec<PayloadSender>
}

impl ServiceFactory for IngestServiceFactory {
    fn new(&self) -> anyhow::Result<Box<dyn Service>> {
        let service = IngestService::new(
            &self.config.clone(), 
            self.subscribers.clone(),
        )?;

        Ok(Box::new(service))
    }
}

#[derive(Debug)]
pub struct IngestService {
    // service lifecycle
    errors: Vec<ServiceError>,

    // config
    read_timeout: Duration,

    // internals
    subscribers: Vec<PayloadSender>,
    listener:    UnixListener,
    state:       State,
}

impl IngestService {
    pub fn new(config: &Arc<Config>, subscribers: Vec<PayloadSender>) -> anyhow::Result<Self> {
        if let Err(e) = fs::remove_file(&config.socket_path) {
            if e.kind() != io::ErrorKind::NotFound {
                return Err(e.into());
            }
        }

        let read_timeout = config.ingest_read_timeout;

        let listener = UnixListener::bind(&config.socket_path)?;
        listener.set_nonblocking(true)?;

        Ok(Self { 
            errors: Vec::new(),
            read_timeout,
            subscribers, 
            listener, 
            state: State::FindConnection 
        })
    }
}

impl Service for IngestService {
    fn cycle(&mut self) -> CycleStatus {
        let state = mem::take(&mut self.state);

        // println!("state: {:?}", &state);

        let result = match state {
            State::FindConnection        => self.find_connection(),
            State::ReadLength(stream)    => self.read_length(stream),
            State::ReadBody(stream, len) => self.read_body(stream, len),
        };

        match result {
            Ok(state) => {
                self.state = state;
                CycleStatus::Healthy
            }
            Err(e) => CycleStatus::Abort(e.into()),
        }
    }

    fn shutdown(&mut self) -> CycleStatus {
        let state = mem::take(&mut self.state);

        if let State::ReadBody(stream, len) = state {
            _ = self.read_body(stream, len);
        };

        return CycleStatus::Healthy;
    }

    fn take_errors(&mut self) -> Vec<ServiceError> {
        mem::take(&mut self.errors)
    }
}

impl IngestService {
    fn find_connection(&self) -> io::Result<State> {
        match self.listener.accept() {
            Ok((stream, _)) => {
                stream.set_read_timeout(Some(self.read_timeout))?;
                Ok(State::ReadLength(stream))
            },

            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(2000));
                Ok(State::FindConnection)
            }

            Err(e) => Err(e.into()), 
        }
    }

    fn read_length(&self, mut stream: UnixStream) -> io::Result<State> {
        _ = self;
        
        let mut buf = [0u8; size_of::<EventSize>()];
        if let Err(e) = stream.read_exact(&mut buf) {
            if e.kind() == ErrorKind::UnexpectedEof {
                // connection lost
                return Ok(State::FindConnection);
            }

            if e.kind() == ErrorKind::TimedOut {
                return Ok(State::ReadLength(stream));
            }

            return Err(e.into());
        }

        let len = u16::from_le_bytes(buf) as usize;
        Ok(State::ReadBody(stream, len))
    }

    fn read_body(&mut self, mut stream: UnixStream, len: usize) -> io::Result<State> {
        let mut buf = [0u8; FRAME_SIZE_MAX];
        if let Err(e) = stream.read_exact(&mut buf[0..len]) {
            if e.kind() == ErrorKind::UnexpectedEof {
                // connection lost
                return Ok(State::FindConnection);
            }

            if e.kind() == ErrorKind::TimedOut {
                return Ok(State::ReadBody(stream, len));
            }

            return Err(e);
        }

        let data = &buf[0..len];

        // if data is malformed we are out of sync with the stream
        if Event::decode(data).is_none() {
            self.errors.push(ServiceError::new(
                ServiceErrorSeverity::Low,
                "Received malformed data. Discarding connection!", 
            ));
            return Ok(State::FindConnection);
        };

        let message: Arc<Vec<u8>> = Arc::new(data.to_vec());

        // forward message to all subscribers
        for subscriber in self.subscribers.as_slice() {
            if let Err(TrySendError::Full(_)) = subscriber.try_send(message.clone()) {
                self.errors.push(ServiceError::new(
                    ServiceErrorSeverity::Medium,
                    "Failed to send: Channel Full!", 
                ));
            };
        }

        return Ok(State::ReadLength(stream));
    }
}