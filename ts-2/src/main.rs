use std::{convert::Infallible, io};

use arrow::ipc::writer::StreamWriter;
use axum::{
    Router, body::Body, 
    http::header, 
    response::{IntoResponse, Response}, 
    routing::get
};

use futures::stream;
use tokio::signal;
use duckdb::Connection;

// /machines/
// /machines/scales_ff01/
// /machines/scales_ff01/live
// /machines/scales_ff01/weights

#[tokio::main]
async fn main() -> io::Result<()> {
    let app = Router::new()
        .route("/machines", get(|| async { "[scales_ff01, scales_ff02]" }))
        .route("/data", get(query_db))
        ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    // tokio::fs::remove_file("/tmp/telemetry.sock").await.unwrap();
    // let listener = tokio::net::UnixListener::bind("/tmp/telemetry.sock").unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
}

const DB_PATH: &str = "/home/entity/work/qitech/prototype-ff01/testing/sandbox/data.db";

async fn query_db() -> impl IntoResponse {
    let batches = tokio::task::spawn_blocking(move || {
        let conn = Connection::open(DB_PATH).unwrap();
        let mut stmt = conn.prepare("SELECT * FROM weights LIMIT 1000;").unwrap();
        stmt.query_arrow([]).unwrap().collect::<Vec<_>>()
    })
    .await
    .unwrap();

    let schema = batches[0].schema();
    let stream = stream::iter(batches.into_iter().map(move |batch| {
        let mut buffer = Vec::new();
        {
            let mut writer = StreamWriter::try_new(&mut buffer, &schema).unwrap();
            writer.write(&batch).unwrap();
            writer.finish().unwrap();
        }
        Ok::<_, Infallible>(buffer)
    }));

    Response::builder()
        .header(header::CONTENT_TYPE, "application/vnd.apache.arrow.stream")
        .body(Body::from_stream(stream))
        .unwrap()
}

async fn shutdown_signal() {
    let signal = signal::unix::SignalKind::terminate();

    signal::unix::signal(signal)
            .expect("failed to install signal handler")
            .recv().await;
}