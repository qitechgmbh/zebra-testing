use std::io;
use bytes::Bytes;
use tokio::{
    io::AsyncWriteExt, 
    net::TcpStream, 
    sync::{ broadcast::{ self, error::RecvError } }
};

pub async fn run(
    mut msg_rx: broadcast::Receiver<Bytes>,
    mut stream: TcpStream,
) -> io::Result<()> {
    loop {
        let data = match msg_rx.recv().await {
            Ok(v) => v,
            Err(RecvError::Closed) => {
                return Ok(());
            }
            Err(RecvError::Lagged(count)) => {
                eprintln!("LiveStream lagged behind and lost: {count} messages!");
                continue;
            }
        };

        let len = data.len() as u16;
        stream.write_all(&len.to_le_bytes()).await?;
        stream.write_all(&data).await?;
        stream.flush().await?;
    }
}