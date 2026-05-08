use std::{io::Write, net::TcpStream};

pub fn internal_error(stream: &mut TcpStream, msg: &str) -> anyhow::Result<()> {
    error(stream, 500, "Internal Server Error", msg)
}

pub fn bad_request(stream: &mut TcpStream, msg: &str) -> anyhow::Result<()> {
    error(stream, 400, "Bad Request", msg)
}

pub fn start_arrow_stream(stream: &mut TcpStream) -> anyhow::Result<()> {
    let header = b"\
        HTTP/1.1 200 OK\r\n\
        Content-Type: application/vnd.apache.arrow.stream\r\n\
        Transfer-Encoding: chunked\r\n\
        Connection: keep-alive\r\n\
        \r\n";

    stream.write_all(header)?;
    Ok(())
}

pub fn write_arrow_batch(stream: &mut TcpStream, data: &[u8]) -> anyhow::Result<()> {
    let size = format!("{:X}\r\n", data.len());
    stream.write_all(size.as_bytes())?;
    stream.write_all(data)?;
    stream.write_all(b"\r\n")?;
    Ok(())
}

pub  fn finish_arrow_stream(stream: &mut TcpStream) -> anyhow::Result<()> {
    stream.write_all(b"0\r\n\r\n")?;
    Ok(())
}

fn error(stream: &mut TcpStream, code: u16, reason: &str, msg: &str) -> anyhow::Result<()> {
    let body = msg.as_bytes();
    let len  = body.len().to_string();

    status_line(stream, code, reason)?;
    stream.write_all(b"Content-Type: text/plain\r\n")?;
    stream.write_all(b"Content-Length: ")?;
    stream.write_all(len.as_bytes())?;
    stream.write_all(b"\r\n\r\n")?;
    stream.write_all(body)?;
    Ok(())
}

fn status_line(stream: &mut TcpStream, code: u16, reason: &str) -> anyhow::Result<()> {
    stream.write_all(b"HTTP/1.1 ")?;
    stream.write_all(code.to_string().as_bytes())?;
    stream.write_all(b" ")?;
    stream.write_all(reason.as_bytes())?;
    stream.write_all(b"\r\n")?;
    Ok(())
}