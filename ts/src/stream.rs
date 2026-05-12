use std::{fmt::Debug, io::ErrorKind, time::Duration};

use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufStream}, 
    select, 
    sync::watch, 
    time::timeout
};

use crate::types::ClientExitReason;

pub type UnixStream = Stream<tokio::net::UnixStream>;
pub type TcpStream  = Stream<tokio::net::TcpStream>;

pub struct Stream<R: AsyncRead + Unpin> {
    inner:   BufStream<R>,
    exit_rx: watch::Receiver<()>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Stream<S> {
    pub fn new(stream: S, exit_rx: watch::Receiver<()>) -> Self {
        let inner = BufStream::new(stream);
        Self { inner, exit_rx }
    }

    pub async fn read<'a>(
        &mut self, 
        buf: &'a mut [u8],
        exit_condition: ExitCondition
    ) -> Result<usize, ClientExitReason> {
        let fut = self.inner.read(buf);
        let exec_res = Self::execute(&mut self.exit_rx, exit_condition, fut).await;

        let result = match exec_res {
            Ok(v) => v,
            Err(status) => return Err(status),
        };

        match result {
            Ok(len) => Ok(len),
            Err(e) => Err(ClientExitReason::IoFailure(e)),
        }
    }

    pub async fn read_line<C: Send + Debug>(
        &mut self, 
        exit_condition: ExitCondition
    ) -> Result<String, ClientExitReason> {
        let mut buf = String::new();

        let fut = self.inner.read_line(&mut buf);
        let exec_res = Self::execute(&mut self.exit_rx, exit_condition, fut).await;

        let result = match exec_res {
            Ok(v) => v,
            Err(status) => return Err(status),
        };

        match result {
            Ok(_) => Ok(buf),
            Err(e) => Err(ClientExitReason::IoFailure(e)),
        }
    }

    pub async fn read_exact<C: Send + Debug>(
        &mut self, 
        buf: &mut [u8],
        exit_condition: ExitCondition
    ) -> Result<(), ClientExitReason> {
        let fut = self.inner.read_exact(buf);
        let exec_res = Self::execute(&mut self.exit_rx, exit_condition, fut).await;

        let result = match exec_res {
            Ok(v) => v,
            Err(status) => return Err(status),
        };

        match result {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                Err(ClientExitReason::Disconnected)
            }
            Err(e) => Err(ClientExitReason::IoFailure(e)),
        }
    }

    pub async fn read_u32<C: Send + Debug>(
        &mut self, 
        exit_condition: ExitCondition
    ) -> Result<u32, ClientExitReason> {
        let fut = self.inner.read_u32();
        let exec_res = Self::execute(&mut self.exit_rx, exit_condition, fut).await;

        let result = match exec_res {
            Ok(v) => v,
            Err(status) => return Err(status),
        };

        match result {
            Ok(v) => Ok(v),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                Err(ClientExitReason::Disconnected)
            }
            Err(e) => Err(ClientExitReason::IoFailure(e)),
        }
    }

    async fn execute<F, T>(
        exit_rx: &mut watch::Receiver<()>,
        exit_condition: ExitCondition, 
        f: F
    ) -> Result<T, ClientExitReason>
    where
        F: Future<Output = T>,
    {
        match exit_condition {
            ExitCondition::Timer(duration) => {
                Self::execute_expiring(duration, f).await
            }

            ExitCondition::Shutdown => {
                Self::execute_terminating(exit_rx, f).await
            }
            ExitCondition::ShutdownOrTimer(duration) => {
                Self::execute_terminating_or_expiring(exit_rx, duration, f).await
            }
        }
    }

    async fn execute_expiring<F, T>(
        duration: Duration,
        f: F,
    ) -> Result<T, ClientExitReason>
    where
        F: Future<Output = T>,
    {
        match timeout(duration, f).await {
            Ok(value) => Ok(value),
            Err(_) => Err(ClientExitReason::Timeout),
        }
    }

    async fn execute_terminating<F, T>(
        exit_rx: &mut watch::Receiver<()>,
        f: F,
    ) -> Result<T, ClientExitReason>
    where
        F: Future<Output = T>,
    {
        select! {
            biased;

            _ = exit_rx.changed() => {
                Err(ClientExitReason::Shutdown)
            }

            value = f => { Ok(value) }
        }
    }

    async fn execute_terminating_or_expiring<F, T>(
        exit_rx: &mut watch::Receiver<()>,
        duration: Duration,
        f: F,
    ) -> Result<T, ClientExitReason>
    where

        F: Future<Output = T>,
    {
        select! {
            biased;

            _ = exit_rx.changed() => {
                Err(ClientExitReason::Shutdown)
            }

            res = timeout(duration, f) => { 
                match res {
                    Ok(value) => Ok(value),
                    Err(_) => Err(ClientExitReason::Timeout),
                }
            }
        }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> Stream<S> {
    pub async fn write_all(
        &mut self,
        buf: &[u8],
        exit_condition: ExitCondition
    ) -> Result<(), ClientExitReason> {
        let fut = self.inner.write_all(buf);
        let exec_res = Self::execute(&mut self.exit_rx, exit_condition, fut).await;

        let result = match exec_res {
            Ok(v) => v,
            Err(status) => return Err(status),
        };

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(ClientExitReason::IoFailure(e)),
        }
    }
}

pub enum ExitCondition {
    ShutdownOrTimer(Duration),
    Timer(Duration),
    Shutdown,
}