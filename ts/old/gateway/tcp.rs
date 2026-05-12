use std::sync::Arc;

use tokio::{
    io::{self, AsyncReadExt}, net::TcpStream, sync::mpsc
};
use httparse::{EMPTY_HEADER, Request, Status};

use crate::{config::Config, root::SystemEvent};

pub async fn run(
    sys_tx: mpsc::Sender<SystemEvent>,
    config: Arc<Config>,
    mut stream: TcpStream,
) -> io::Result<()> {
    let mut buf = Vec::new();
    let mut temp = [0u8; 4096];

    #[allow(unused_assignments)]
    let mut headers = [EMPTY_HEADER; 32];

    let request = loop {
        headers = [EMPTY_HEADER; 32];

        let n = stream.read(&mut temp).await?;

        if n == 0 {
            return Ok(());
        }

        buf.extend_from_slice(&temp[..n]);

        let mut req = Request::new(&mut headers);

        let result = req.parse(&buf);

        if let Ok(Status::Complete(_)) = result {
            
        };

        match req.parse(&buf) {
            Ok(Status::Partial) => continue,
            Ok(Status::Complete(_)) => break req,
            Err(_) => continue,
        }
    };

    let Some(method) = request.method else {
        // http::bad_request(&mut stream, responses::MISSING_METHOD)?;
        return Ok(());
    };

    if method != "GET" {
        // http::bad_request(&mut stream, responses::INVALID_METHOD)?;
        return Ok(());
    }

    let Some(path) = request.path else {
        // http::bad_request(&mut stream, responses::MISSING_PATH)?;
        return Ok(());
    };

    let Some(route) = parse_route(path) else {
        // http::bad_request(&mut stream, responses::MISSING_PATH)?;
        return Ok(());
    };

    match route {
        Route::Machine(route) => {
            match route {
                MachineRoute::Live { name } => {

                }

                MachineRoute::MachineQuery { name, table, query } => {
                    _ = name;
                    _ = table;
                    _ = query;
                }
            }
        }

        Route::System(route) => {
            match route {
                SystemRoute::Status => {

                }
            }
        }
    }

    Ok(())
}

pub enum Route<'a> {
    Machine(MachineRoute<'a>),
    System(SystemRoute),
}

pub enum MachineRoute<'a> {
    Live { 
        name: &'a str 
    },
    MachineQuery {
        name:  &'a str,
        table: &'a str,
        query: Option<&'a str>,
    }
}

pub enum SystemRoute {
    Status,
}

pub fn parse_route<'a>(input: &'a str) -> Option<Route<'a>> {
    let (path, query) = match input.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (input, None),
    };

    let mut parts = path.split('/').filter(|p| !p.is_empty());

    match parts.next()? {
        "system" => {
            match parts.next()? {
                "status" => Some(Route::System(SystemRoute::Status)),
                _ => None,
            }
        }

        "machine" => {
            let name = parts.next()?;

            match parts.next()? {
                "live" => Some(Route::Machine(MachineRoute::Live { name })),

                table => Some(Route::Machine(MachineRoute::MachineQuery {
                    name,
                    table,
                    query,
                })),
            }
        }

        _ => None,
    }
}