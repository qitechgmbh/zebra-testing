use actix::prelude::*;
use telemetry_core::Event;
use tokio::{io::AsyncReadExt, net::{UnixListener, UnixStream}, task};

/// Define message
#[derive(Message)]
#[rtype(result = "Result<bool, std::io::Error>")]
struct Ping;

struct IngestListener {
    socket_path: String
}

impl Actor for IngestListener {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let listener = UnixListener::bind(&self.socket_path).unwrap();

        ctx.spawn(
            async move {
                loop {
                    match listener.accept().await {
                        Ok((stream, _)) => {
                            ConnectionActor { stream: Some(stream) }.start();
                        }

                        Err(e) => {
                            eprintln!("Accept Error: {e}");
                            break;
                        }
                    }
                }
            }
            .into_actor(self)
            .map(|_, _, ctx| {
                ctx.stop();
            }),
        );
    }
}

struct ConnectionActor {
    pub stream: Option<UnixStream>,
}

impl Actor for ConnectionActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut stream = self.stream.take().unwrap();

        ctx.spawn(
            async move {
                loop {
                    // read length
                    let mut len_buf = [0u8; 2];

                    if stream.read_exact(&mut len_buf).await.is_err() {
                        break;
                    }

                    let len =
                        u16::from_le_bytes(len_buf) as usize;

                    let mut buf = vec![0u8; len];

                    if stream.read_exact(&mut buf).await.is_err() {
                        break;
                    }

                    if Event::decode(&buf).is_none() {
                        break;
                    }

                    // let payload = Arc::new(buf);
// 
                    // for tx in &subscribers {
                    //     let _ = tx.try_send(payload.clone());
                    // }
                }
            }
            .into_actor(self)
            .map(|_, _, ctx| {
                ctx.stop();
            }),
        );
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let handle = task::spawn(async move {
        loop {
            println!("listener task running");
            tokio::time::sleep(
                std::time::Duration::from_secs(1)
            )
            .await;
        }
    });

    // handle.

    tokio::signal::ctrl_c().await
}



// Service:  holds handle to listener task
// Shutdown: abort the task via the handle

// ServiceRunner runs in a loop waiting for shutdown signal
// When received it stops the task

// Recorder -> Requires graveful shutdown
// WIll need a statemachine...