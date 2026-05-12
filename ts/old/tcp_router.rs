use std::io;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

// TODO: 
// check if machine name exists, if it does, does it have such a table name. live always works

// each schema defines endpoints and bla bla bla -> SpawnLive, SpawnQuery(DBName + SQL query + values)

pub async fn run(
    mut stream: TcpStream,
) -> io::Result<()> {
    use httparse::{EMPTY_HEADER, Request, Status};

    let mut buf = [0u8; 4096];

    loop {
        let size = stream.read(&mut buf).await?;

        if size == 0 {
            // connection closed
            return Ok(());
        }

        let mut headers = [EMPTY_HEADER; 32];
        let mut request = Request::new(&mut headers);

        let status = match request.parse(&buf[..size]) {
            Ok(v) => v,
            Err(e) => {
                let message = format!("Failed to parse request: {e}");
                stream.write_all(message.as_bytes()).await?;
                continue;
            }
        };

        match status {
            Status::Partial => continue,

            Status::Complete(_) => {
                let Some(method) = request.method else {
                    // http::bad_request(&mut stream, responses::MISSING_METHOD)?;
                    continue;
                };

                if method != "GET" {
                    // http::bad_request(&mut stream, responses::INVALID_METHOD)?;
                    continue;
                }

                let Some(path) = request.path else {
                    // http::bad_request(&mut stream, responses::MISSING_PATH)?;
                    continue;
                };

                

                // let (route, query) = utils::destruct_path(path);

                return Ok(());
            }
        }
    }
}