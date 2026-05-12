use std::{fmt::Debug, io::ErrorKind, time::Duration};

use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, BufStream}, 
    select, 
    sync::watch, 
    time::timeout
};

use crate::types::CompleteReason;

pub struct Stream<R: AsyncRead + Unpin> {
    inner:   BufStream<R>,
    exit_rx: watch::Receiver<()>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Stream<S> {
    pub fn new(stream: S, exit_rx: watch::Receiver<()>) -> Self {
        let inner = BufStream::new(stream);
        Self { inner, exit_rx }
    }

    pub async fn read_line<C: Send + Debug>(
        &mut self, 
        exit_condition: ExitCondition
    ) -> Result<String, CompleteReason<C>> {
        let mut buf = String::new();

        let fut = self.inner.read_line(&mut buf);
        let exec_res = Self::execute(&mut self.exit_rx, exit_condition, fut).await;

        let result = match exec_res {
            Ok(v) => v,
            Err(status) => return Err(status),
        };

        match result {
            Ok(_) => Ok(buf),
            Err(e) => Err(CompleteReason::IoFailure(e)),
        }
    }

    pub async fn read_exact<C: Send + Debug>(
        &mut self, 
        buf: &mut [u8],
        exit_condition: ExitCondition
    ) -> Result<(), CompleteReason<C>> {
        let fut = self.inner.read_exact(buf);
        let exec_res = Self::execute(&mut self.exit_rx, exit_condition, fut).await;

        let result = match exec_res {
            Ok(v) => v,
            Err(status) => return Err(status),
        };

        match result {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                Err(CompleteReason::ClientDisconnected)
            }
            Err(e) => Err(CompleteReason::IoFailure(e)),
        }
    }

    pub async fn read_u32<C: Send + Debug>(
        &mut self, 
        exit_condition: ExitCondition
    ) -> Result<u32, CompleteReason<C>> {
        let fut = self.inner.read_u32();
        let exec_res = Self::execute(&mut self.exit_rx, exit_condition, fut).await;

        let result = match exec_res {
            Ok(v) => v,
            Err(status) => return Err(status),
        };

        match result {
            Ok(v) => Ok(v),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                Err(CompleteReason::ClientDisconnected)
            }
            Err(e) => Err(CompleteReason::IoFailure(e)),
        }
    }

    async fn execute<C, F, T>(
        exit_rx: &mut watch::Receiver<()>,
        exit_condition: ExitCondition, 
        f: F
    ) -> Result<T, CompleteReason<C>>
    where
        C: Debug + Send,
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

    async fn execute_expiring<C, F, T>(
        duration: Duration,
        f: F,
    ) -> Result<T, CompleteReason<C>>
    where
        C: Debug + Send,
        F: Future<Output = T>,
    {
        match timeout(duration, f).await {
            Ok(value) => Ok(value),
            Err(_) => Err(CompleteReason::Timeout),
        }
    }

    async fn execute_terminating<C, F, T>(
        exit_rx: &mut watch::Receiver<()>,
        f: F,
    ) -> Result<T, CompleteReason<C>>
    where
        C: Debug + Send,
        F: Future<Output = T>,
    {
        select! {
            biased;

            _ = exit_rx.changed() => {
                Err(CompleteReason::Shutdown)
            }

            value = f => { Ok(value) }
        }
    }

    async fn execute_terminating_or_expiring<C, F, T>(
        exit_rx: &mut watch::Receiver<()>,
        duration: Duration,
        f: F,
    ) -> Result<T, CompleteReason<C>>
    where
        C: Debug + Send,
        F: Future<Output = T>,
    {
        select! {
            biased;

            _ = exit_rx.changed() => {
                Err(CompleteReason::Shutdown)
            }

            res = timeout(duration, f) => { 
                match res {
                    Ok(value) => Ok(value),
                    Err(_) => Err(CompleteReason::Timeout),
                }
            }
        }
    }
}

pub enum ExitCondition {
    ShutdownOrTimer(Duration),
    Timer(Duration),
    Shutdown,
}