pub mod receiver;
pub mod settings;

use clap::Parser;
use settings::Settings;
use tokio::sync::mpsc;
use receiver::start_receiver;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let settings = Settings::parse();

    tracing::info!("Starting rsudp-rust on port {}", settings.port);

    // Create channel
    let (tx, mut rx) = mpsc::channel(100);

    // Spawn receiver
    let port = settings.port;
    tokio::spawn(async move {
        if let Err(e) = start_receiver(port, tx).await {
            tracing::error!("Receiver error: {}", e);
        }
    });

    // Consumer loop
    while let Some(packet) = rx.recv().await {
        tracing::info!("Processed packet from {}: {} bytes", packet.source, packet.data.len());
    }
}