use std::net::SocketAddr;
use std::time::Instant;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct Packet {
    pub source: SocketAddr,
    pub data: Vec<u8>,
    pub received_at: Instant,
}

pub async fn start_receiver(port: u16, tx: mpsc::Sender<Packet>) -> std::io::Result<()> {
    // T009: Bind socket
    let addr = format!("0.0.0.0:{}", port);
    let socket = UdpSocket::bind(&addr).await?;
    info!("Listening on {}", socket.local_addr()?);

    let mut buf = [0u8; 65535]; // Max UDP size

    // T010: Reception loop
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, source)) => {
                let data = buf[..len].to_vec();
                let received_at = Instant::now();

                info!("Received {} bytes from {}", len, source);

                let packet = Packet {
                    source,
                    data,
                    received_at,
                };

                // T011: Send to channel
                if let Err(e) = tx.send(packet).await {
                    info!("Receiver channel closed: {}", e);
                    break;
                }
            }
            Err(e) => {
                warn!("Failed to receive packet: {}", e);
                // Continue loop
            }
        }
    }

    Ok(())
}
