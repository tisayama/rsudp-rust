use rsudp_rust::pipeline::{run_pipeline};
use rsudp_rust::trigger::{TriggerConfig};
use rsudp_rust::intensity::{IntensityConfig};
use rsudp_rust::web::{WebState};
use tokio::sync::mpsc;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: Option<String>,

    #[arg(short, long, default_value = "ENE,ENN,ENZ")]
    channels: String,

    #[arg(short, long, default_value_t = 8081)]
    web_port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let web_state = WebState::new();
    let (pipe_tx, pipe_rx) = mpsc::channel(100);

    // 1. Start Web Server
    let addr = format!("0.0.0.0:{}", args.web_port);
    let app_state = web_state.clone();
    tokio::spawn(async move {
        let router = rsudp_rust::web::routes::create_router(app_state).await;
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        tracing::info!("WebUI server listening on {}", addr);
        axum::serve(listener, router).await.unwrap();
    });

    // 2. Setup Configs
    let trigger_config = TriggerConfig {
        sta_sec: 1.0,
        lta_sec: 10.0,
        threshold: 3.0,
        reset_threshold: 1.5,
    };

    let parts: Vec<String> = args.channels.split(',').map(|s| s.to_string()).collect();
    let intensity_config = if parts.len() == 3 {
        Some(IntensityConfig {
            channels: parts,
            sample_rate: 100.0,
            sensitivities: vec![1.0 / 384500.0, 1.0 / 384500.0, 1.0 / 384500.0],
        })
    } else {
        None
    };

    // 3. Simulation or UDP mode
    if let Some(path) = args.file {
        tracing::info!("Simulation mode: processing file {}", path);
        
        let ws = web_state.clone();
        let pipeline_handle = tokio::spawn(async move {
            run_pipeline(pipe_rx, trigger_config, intensity_config, ws).await;
        });

        // Load the file and feed it to the pipeline
        // MiniSEED files consist of multiple 512-byte records
        let bytes = std::fs::read(&path).unwrap();
        for chunk in bytes.chunks(512) {
            let _ = pipe_tx.send(chunk.to_vec()).await;
        }
        
        // Give some time for pipeline to finish
        drop(pipe_tx);
        let _ = pipeline_handle.await;
        
        tracing::info!("Simulation complete.");
        return;
    }

    // Keep main alive only in UDP mode
    tokio::signal::ctrl_c().await.unwrap();
}