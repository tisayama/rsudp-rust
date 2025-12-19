pub mod receiver;
pub mod settings;
pub mod trigger;
pub mod parser;
pub mod pipeline;

use clap::Parser;
use settings::Settings;
use tokio::sync::mpsc;
use receiver::start_receiver;
use pipeline::{FilterManager, run_pipeline};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let settings = Settings::parse();

    // Channel to Pipeline (MiniSEED bytes)
    let (pipe_tx, pipe_rx) = mpsc::channel(100);
    let manager = FilterManager::new(100, 1000);

    // Spawn Ingestion Pipeline task
    let pipeline_handle = tokio::spawn(async move {
        run_pipeline(pipe_rx, manager).await;
    });

    if !settings.file.is_empty() {
        // --- Simulation Mode ---
        tracing::info!("Simulation mode: processing {} files", settings.file.len());
        
        for path in settings.file {
            match std::fs::read(&path) {
                Ok(bytes) => {
                    tracing::info!("Feeding file: {}", path);
                    if let Err(e) = pipe_tx.send(bytes).await {
                        tracing::error!("Pipeline channel error: {}", e);
                        break;
                    }
                }
                Err(e) => tracing::error!("Failed to read file {}: {}", path, e),
            }
        }
        
        // Drop tx to signal end of stream to pipeline
        drop(pipe_tx);
        // Wait for pipeline to complete processing
        let _ = pipeline_handle.await;
        tracing::info!("Simulation complete.");
        
    } else {
        // --- Real-time UDP Mode ---
        tracing::info!("Starting rsudp-rust on port {}", settings.port);

        let (net_tx, mut net_rx) = mpsc::channel(100);
        let port = settings.port;

        // Spawn UDP Receiver
        tokio::spawn(async move {
            if let Err(e) = start_receiver(port, net_tx).await {
                tracing::error!("Receiver error: {}", e);
            }
        });

        // Forward network data to pipeline
        while let Some(packet) = net_rx.recv().await {
            if let Err(e) = pipe_tx.send(packet.data).await {
                tracing::error!("Pipeline channel error: {}", e);
                break;
            }
        }
    }
}