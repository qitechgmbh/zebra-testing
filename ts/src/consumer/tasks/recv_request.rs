use httparse::{EMPTY_HEADER, Request};

use crate::{consumer::{MachineRoute, Route, SystemRoute}, stream::{ExitCondition, TcpStream}, types::{ClientExitReason, ClientTaskResult}};
use super::super::{types::RecvRequestError};

pub async fn run(mut stream: TcpStream) -> ClientTaskResult<Result<Route, RecvRequestError>> {
    let mut buf = [0u8; 4096];

    loop {
        let len = stream.read(&mut buf, ExitCondition::Shutdown).await?;

        if len == 0 {
            // connection closed
            return Err(ClientExitReason::Disconnected);
        }

        let mut headers = [EMPTY_HEADER; 64];
        let mut request = Request::new(&mut headers);

        let status = match request.parse(&buf[..len]) {
            Ok(v) => v,
            Err(e) => {
                return Ok(Err(RecvRequestError::HttpParse(e)));
            }
        };

        if status.is_partial() {
            continue;
        }

        if let httparse::Status::Partial = status {
            continue;
        }

        let Some(method) = request.method else {
            return Ok(Err(RecvRequestError::MethodMissingg));
        };

        if method != "GET" {
            return Ok(Err(RecvRequestError::MethodUnsupported));
        }

        let Some(path) = request.path else {
            return Ok(Err(RecvRequestError::PathMissing));
        };

        if let Some(route) = parse_route(path) {
            return Ok(Ok(route));
        } else {
            return Ok(Err(RecvRequestError::UnknownRoute));
        }
    };
}

pub fn parse_route(input: & str) -> Option<Route> {
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
            let name = parts.next()?.to_string();

            match parts.next()? {
                "live" => Some(Route::Machine(MachineRoute::Live { name })),

                table => Some(Route::Machine(MachineRoute::Query {
                    name,
                    table: table.to_string(),
                    query: query.map(|v| v.to_string()),
                })),
            }
        }

        _ => None,
    }
}