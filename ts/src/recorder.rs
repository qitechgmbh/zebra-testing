use tokio::{io::AsyncReadExt, net::TcpStream, sync::watch};

use crate::system::IOTaskState;



pub async fn run(
    state_tx: watch::Sender<IOTaskState>,
    mut stream: TcpStream, 
) -> anyhow::Result<()> {
    let mut buf = [0u8; 4096];

    state_tx.send(IOTaskState::Idle)?;

    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut request = httparse::Request::new(&mut headers);

    loop {
        let size = stream.read(&mut buf).await?;

        if size == 0 {
            // connection closed
            return Ok(());
        }

        state_tx.send(IOTaskState::Processing)?;

        // match request.parse(&buf[..size])? {
        // 
        // }
    }
}