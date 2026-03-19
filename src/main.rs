use std::{net::UdpSocket, time::{Duration, Instant}};

mod xtrem;
use xtrem::*;

fn main() {
    let (sock_rx, sock_tx) = setup_sockets(5555, "192.168.4.255:4444");

    let device_ids = [0x01, 0x02];
    let cmds: Vec<Vec<u8>> = device_ids
        .iter()
        .map(|&id| build_request(id))
        .collect();

    loop {
        // Send requests
        send_requests(&sock_tx, &cmds);

        // Gather replies
        let (total_weight, count) = collect_responses(&sock_rx, &device_ids);

        println!("Data: {:?} | {:?}", total_weight, count);

        std::thread::sleep(Duration::from_millis(300));
    }
}

fn setup_sockets(rx_port: u16, tx_addr: &str) -> (UdpSocket, UdpSocket)
{
    let sock_rx = UdpSocket::bind(("0.0.0.0", rx_port)).unwrap();
    let _ = sock_rx.set_nonblocking(true);

    let sock_tx = UdpSocket::bind("0.0.0.0:0").unwrap();
    sock_tx.set_broadcast(true).unwrap();
    sock_tx.connect(tx_addr).unwrap();

    (sock_rx, sock_tx)
}

/// Helper: Build frame for a device ID
fn build_request(dest_id: u8) -> Vec<u8> {
    let request = XtremRequest {
        id_origin: 0x00,
        id_dest: dest_id,
        data_address: DataAddress::Weight,
        function: Function::ReadRequest,
        data: Vec::new(),
    };
    let frame: Frame = request.into();
    frame.as_bytes()
}

/// Helper: Send requests
fn send_requests(sock_tx: &UdpSocket, cmds: &[Vec<u8>]) {
    for cmd in cmds {
        sock_tx.send(cmd).unwrap();
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn collect_responses(sock_rx: &UdpSocket, device_ids: &[u8]) -> (f64, usize)
{
    let start = Instant::now();
    let timeout = Duration::from_millis(300);

    let mut buf = [0u8; 2048];
    let mut total_weight = 0.0;
    let mut received_count = 0;

    while start.elapsed() < timeout && received_count < device_ids.len() {
        match sock_rx.recv(&mut buf) {
            Ok(n) => {
                if let Some((id, weight)) = parse_response(&buf[..n]) {
                    println!("Received weight {weight} from ID {id}");
                    total_weight += weight;
                    received_count += 1;
                } else {
                    println!("Failed to parse response...");
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                println!("Err: [XTREM] Socket error: {:?}", e);
                break;
            }
        }
    }
    (total_weight, received_count)
}

/// Helper: Parse a single response
fn parse_response(buf: &[u8]) -> Option<(u8, f64)> {
    let clean: String = buf
        .iter()
        .filter(|b| b.is_ascii_graphic() || **b == b' ')
        .map(|&b| b as char)
        .collect();

    if clean.len() < 2 {
        return None;
    }

    let id_str = &clean[0..2];
    if let std::result::Result::Ok(id) = id_str.parse::<u8>() {
        let weight = Frame::parse_weight_from_response(buf);
        Some((id, weight))
    } else {
        println!("Failed to parse ID from '{id_str}'");
        None
    }
}