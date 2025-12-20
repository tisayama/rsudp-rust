use std::net::UdpSocket;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn run_streamer_test(speed: f64) -> bool {
    // 1. Setup UDP receiver
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind");
    let addr = socket.local_addr().expect("Failed to get local addr");
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("Failed to set timeout");

    // 2. Run streamer
    let mut child = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "streamer",
            "--",
            "--file",
            "/home/tisayama/Development/rustrsudp_speckit/references/mseed/fdsnws.mseed",
            "--addr",
            &addr.to_string(),
            "--speed",
            &speed.to_string(),
        ])
        .current_dir("rsudp-rust")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start streamer");

    // 3. Receive at least one packet
    let mut buf = [0u8; 65535];
    let start = Instant::now();
    let mut received = false;

    while start.elapsed() < Duration::from_secs(10) {
        if let Ok((len, _)) = socket.recv_from(&mut buf) {
            if len > 0 {
                let data = String::from_utf8_lossy(&buf[..len]);
                if data.starts_with("[") && data.contains("EHZ") {
                    received = true;
                    break;
                }
            }
        }
    }

    let _ = child.kill();
    received
}

#[test]
fn test_streamer_realtime() {
    assert!(run_streamer_test(1.0), "Real-time streaming failed");
}

#[test]
fn test_streamer_fast() {
    assert!(run_streamer_test(10.0), "10x speed streaming failed");
}

#[test]
fn test_streamer_ultra_fast() {
    assert!(run_streamer_test(100.0), "100x speed streaming failed");
}
