use chrono::{DateTime, Utc};
use clap::Parser;
use rsudp_rust::parser::header::parse_header;
use rsudp_rust::parser::mseed::parse_single_record;
use serde_json::json;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::net::{SocketAddr, UdpSocket};
use std::path::PathBuf;
use tokio::select;
use tokio::time::{Duration, Instant, sleep};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the source MiniSEED file
    #[arg(short, long)]
    file: PathBuf,

    /// UDP destination address (IP:Port)
    #[arg(short, long, default_value = "127.0.0.1:12345")]
    addr: SocketAddr,

    /// Playback speed multiplier (1.0 = real-time)
    #[arg(short, long, default_value_t = 1.0)]
    speed: f64,

    /// Whether to restart from the beginning when the end is reached
    #[arg(short, long, default_value_t = false)]
    r#loop: bool,
}

#[derive(Debug, Clone)]
struct RecordIndexEntry {
    start_time: DateTime<Utc>,
    file_offset: u64,
}

fn index_mseed_file(path: &PathBuf) -> Result<Vec<RecordIndexEntry>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut index = Vec::new();
    let mut buffer = [0u8; 512];
    let mut offset = 0u64;

    while file.read_exact(&mut buffer).is_ok() {
        if let Ok(header) = parse_header(&buffer) {
            index.push(RecordIndexEntry {
                start_time: header.starttime,
                file_offset: offset,
            });
        }
        offset += 512;
    }

    index.sort_by_key(|e| e.start_time);
    Ok(index)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    tracing::info!("Starting UDP MiniSEED Streamer");
    tracing::info!("File: {:?}", args.file);
    tracing::info!("Target: {}", args.addr);
    tracing::info!("Speed: {}x", args.speed);
    tracing::info!("Loop mode: {}", args.r#loop);

    tracing::info!("Indexing file...");
    let index = index_mseed_file(&args.file)?;
    let total_records = index.len();
    tracing::info!("Indexed {} records", total_records);

    if index.is_empty() {
        tracing::error!("No valid MiniSEED records found in file");
        return Ok(());
    }

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let mut file = File::open(&args.file)?;
    let mut buffer = [0u8; 512];

    let stop = tokio::signal::ctrl_c();
    tokio::pin!(stop);

    'outer: loop {
        let session_start_real = Instant::now();
        let session_start_data = index[0].start_time;
        let total_data_duration =
            (index.last().unwrap().start_time - session_start_data).to_std()?;

        for (i, entry) in index.iter().enumerate() {
            let data_elapsed = (entry.start_time - session_start_data).to_std()?;
            let real_wait = data_elapsed.div_f64(args.speed);

            let target_time = session_start_real + real_wait;
            let now = Instant::now();

            if target_time > now {
                select! {
                    _ = sleep(target_time - now) => {},
                    _ = &mut stop => {
                        tracing::info!("Shutdown signal received");
                        break 'outer;
                    }
                }
            }

            file.seek(SeekFrom::Start(entry.file_offset))?;
            file.read_exact(&mut buffer)?;

            match parse_single_record(&buffer) {
                Ok(segment) => {
                    let ts_f64 = segment.starttime.timestamp() as f64
                        + (segment.starttime.timestamp_subsec_nanos() as f64 / 1_000_000_000.0);
                    let mut packet_data = vec![json!(segment.channel), json!(ts_f64)];
                    for &sample in &segment.samples {
                        packet_data.push(json!(sample as i32));
                    }

                    let json_packet = serde_json::to_string(&packet_data)?;
                    socket.send_to(json_packet.as_bytes(), args.addr)?;

                    if i % 100 == 0 || i == total_records - 1 {
                        let percent = (i as f64 / total_records as f64) * 100.0;
                        let eta = if args.speed > 0.0 {
                            let remaining_data = total_data_duration
                                .checked_sub(data_elapsed)
                                .unwrap_or(Duration::ZERO);
                            format!("{:?}", remaining_data.div_f64(args.speed))
                        } else {
                            "N/A".to_string()
                        };
                        tracing::info!(
                            "[{:.1}%] Sent record {}/{}. ETA: {}",
                            percent,
                            i + 1,
                            total_records,
                            eta
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to parse record at offset {}: {}",
                        entry.file_offset,
                        e
                    );
                }
            }
        }

        if !args.r#loop {
            break;
        }
        tracing::info!("Looping back to start");
    }

    tracing::info!("Streamer finished");
    Ok(())
}
