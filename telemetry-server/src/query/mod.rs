use std::{io::Read, net::{TcpListener, TcpStream}, sync::Arc, thread};

use duckdb::Connection;
use httparse::{EMPTY_HEADER, Request, Status};

mod utils;
mod http;
mod sql;
mod args;
use args::QueryArgs;
mod responses;

use crate::{config::Config, query::utils::respond_arrow};

pub fn run(config: Arc<Config>) -> anyhow::Result<()> {
    let address = format!("0.0.0.0:{}", config.query_port);
    let listener = TcpListener::bind(address)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let config = config.clone();
                thread::spawn(move || handle_client(config, stream));
            }
            Err(e) => eprintln!("Failed to accept client: {e}"),
        }
    }

    Ok(())
}

fn handle_client(config: Arc<Config>, mut stream: TcpStream) -> anyhow::Result<()> {
    let mut buf = [0u8; 4096];

    loop {
        let size = stream.read(&mut buf)?;

        if size == 0 {
            // connection closed
            return Ok(());
        }

        let mut headers = [EMPTY_HEADER; 32];
        let mut request = Request::new(&mut headers);

        match request.parse(&buf[..size])? {
            Status::Complete(_) => {
                let Some(method) = request.method else {
                    http::bad_request(&mut stream, responses::MISSING_METHOD)?;
                    continue;
                };

                if method != "GET" {
                    http::bad_request(&mut stream, responses::INVALID_METHOD)?;
                    continue;
                }

                let Some(path) = request.path else {
                    http::bad_request(&mut stream, responses::MISSING_PATH)?;
                    continue;
                };

                let (route, query) = utils::destruct_path(path);

                let args = match QueryArgs::new(query.unwrap_or("")) {
                    Ok(v) => v,
                    Err(e) => {
                        http::bad_request(&mut stream, &e.to_string())?;
                        continue;
                    }
                };
                
                let result = match route {
                    "/weights" => sql::create("weights", utils::weights_fields(), args),
                    "/plates"  => sql::create("plates",  utils::plates_fields(), args),
                    "/orders"  => sql::create("orders",  utils::orders_fields(), args),
                    "/logs"    => sql::create("logs",    utils::logs_fields(), args),
                    _ => {
                        http::bad_request(&mut stream, responses::UNSUPPORTED_ROUTE)?;
                        continue;
                    }
                };

                let (sql, params) = match result {
                    Ok(v) => v,
                    Err(e) => {
                        http::internal_error(&mut stream, &e.to_string())?;
                        continue;
                    },
                };

                let connection = match Connection::open(&config.db_path) {
                    Ok(v) => v,
                    Err(e) => {
                        http::internal_error(&mut stream, &e.to_string())?;
                        continue;
                    },
                };

                let statement = match connection.prepare(&sql) {
                    Ok(v) => v,
                    Err(e) => {
                        http::internal_error(&mut stream, &e.to_string())?;
                        continue;
                    },
                };

                if let Err(e) = respond_arrow(statement, &mut stream, params) {
                    http::internal_error(&mut stream, &e.to_string())?;
                    continue;
                }
            }

            Status::Partial => continue
        }
    }
}