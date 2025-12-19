pub mod receiver;
pub mod settings;
pub mod trigger;
pub mod parser;
pub mod pipeline;
pub mod web;

use clap::Parser;
use settings::Settings;
use tokio::sync::{mpsc, broadcast};
use receiver::start_receiver;
use pipeline::run_pipeline;
use std::sync::{Arc, RwLock};
use web::{PlotSettings, stream::WebState};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let settings = Settings::parse();

    // Setup WebUI State
    let (ws_tx, _) = broadcast::channel(100);
    let web_settings = Arc::new(RwLock::new(PlotSettings {
        active_channels: vec!["SHZ".to_string(), "EHZ".to_string()],
        window_seconds: 60,
        auto_scale: true,
        theme: "dark".to_string(),
    }));
    
    let web_state = Arc::new(WebState {
        settings: web_settings.clone(),
        tx: ws_tx.clone(),
    });

    // Start WebUI Server
    let app_state = web_state.clone();
    tokio::spawn(async move {
        let router = web::routes::create_router(app_state).await;
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
        tracing::info!("WebUI server listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap();
    });

    if settings.mock {
        let ws_tx_mock = ws_tx.clone();
        tokio::spawn(async move {
            tracing::info!("Starting mock data producer for stress testing...");
            web::test_utils::start_mock_producer(ws_tx_mock).await;
        });
    }

    // Channel to Pipeline (MiniSEED bytes)
    let (pipe_tx, pipe_rx) = mpsc::channel(100);
    let manager = pipeline::PipelineManager::new(ws_tx);

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

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown...");
}