use std::net::UdpSocket;
use std::time::{Duration, Instant};

fn main() -> std::io::Result<()> {
    let rx_port = 5555; // scale -> PC
    let tx_addr = "192.168.4.255:4444"; // PC -> scale

    // CREATE RX SOCKET
    let sock_rx = UdpSocket::bind(("0.0.0.0", rx_port))?;
    sock_rx.set_nonblocking(true)?;
    println!("[RX] Listening on {}", rx_port);

    // CREATE TX SOCKET
    let sock_tx = UdpSocket::bind("0.0.0.0:0")?;                                                                                                  
    sock_tx.set_broadcast(true)?;
    sock_tx.connect(tx_addr)?;
    println!("[TX] Sending to {}", tx_addr);

    // ---------- BUILD FRAME ----------
    let mut cmd: Vec<u8> = Vec::new();

    cmd.push(0x02); // STX
    cmd.extend_from_slice(b"00"); // ID_O  (PC)
    cmd.extend_from_slice(b"02"); // ID_D  (scale address)
    cmd.extend_from_slice(b"R"); // F = 'W' (write) 'R' (read)
    cmd.extend_from_slice(b"0000"); // D_ADDRESS (register 0020)
    cmd.extend_from_slice(b"00"); // D_L = "02" (two ASCII data bytes)
    // cmd.extend_from_slice(b"02"); // DATA = "02" (ASCII '0','2')

    // Compute LRC (XOR of all bytes from ID_O through end of DATA)
    let lrc = cmd[1..].iter().fold(0u8, |acc, &b| acc ^ b);
    cmd.extend_from_slice(format!("{:02X}", lrc).as_bytes()); // LRC as ASCII hex
    cmd.push(0x03); // ETX
    cmd.extend_from_slice(b"\r\n"); // CRLF terminator
    // --------------------------------

    let cmd = cmd.as_slice();

    println!("[TX] Sending...");
    println!("[TX] HEX  : {:02X?}", &cmd);
    // sock_tx.send(cmd)?;

    loop {
        println!("[TX] ASCII  : {}", String::from_utf8_lossy(cmd));
        sock_tx.send(cmd)?;

        let start = Instant::now();
        let timeout = Duration::from_millis(300);
        let mut buf = [0u8; 2048];

        loop {
            match sock_rx.recv(&mut buf) {
                Ok(n) => {
                    println!("\n[RX] {} bytes", n);
                    println!("HEX  : {:02X?}", &buf[..n]);
                    println!("ASCII: {}", String::from_utf8_lossy(&buf[..n]));
                    break;
                }
                Err(_) if start.elapsed() < timeout => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => {
                    println!("[RX] Timeout (no reply)");
                    break;
                }
            }
        }

        std::thread::sleep(Duration::from_millis(500));
    }
}
