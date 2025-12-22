use rsudp_rust::pipeline::run_pipeline;
use rsudp_rust::trigger::TriggerConfig;
use rsudp_rust::intensity::IntensityConfig;
use rsudp_rust::web::WebState;
use rsudp_rust::receiver::start_receiver;
use rsudp_rust::parser::stationxml::fetch_sensitivity;
use rsudp_rust::parser::mseed::parse_mseed_file;
use tokio::sync::mpsc;
use clap::Parser;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// MiniSEED file for simulation
    #[arg(short, long)]
    file: Option<String>,

    /// Network code (used if no file is provided)
    #[arg(short, long, default_value = "AM")]
    network: String,

    /// Station name (used if no file is provided)
    #[arg(short, long, default_value = "S9AF3")]
    station: String,

    /// Channels for intensity calculation (must be 3)
    #[arg(short, long, default_value = "ENE,ENN,ENZ")]
    channels: String,

    /// WebUI port
    #[arg(short, long, default_value_t = 8081)]
    web_port: u16,

    /// UDP port for receiver
    #[arg(short, long, default_value_t = 12345)]
    udp_port: u16,

    /// Seconds of data to display in plot
    #[arg(long, default_value_t = 90.0)]
    window_seconds: f64,

    /// Ratio of duration to wait before posting (0.0 to 1.0)
    #[arg(long, default_value_t = 0.7)]
    save_pct: f64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let web_state = WebState::new();
    {
        let mut settings = web_state.settings.write().unwrap();
        settings.window_seconds = args.window_seconds;
        settings.save_pct = args.save_pct;
    }
    
    // Update default history settings as well
    {
        let mut history = web_state.history.lock().unwrap();
        let mut h_settings = history.get_settings();
        h_settings.save_pct = args.save_pct;
        history.update_settings(h_settings);
    }

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

    // 2. Determine target station and fetch metadata
    let (net, sta) = if let Some(path) = &args.file {
        // Peek at file to find station
        match parse_mseed_file(path) {
            Ok(segs) if !segs.is_empty() => (segs[0].network.clone(), segs[0].station.clone()),
            _ => (args.network.clone(), args.station.clone()),
        }
    } else {
        (args.network.clone(), args.station.clone())
    };

    tracing::info!("Using metadata for Station: {}.{}", net, sta);
    let sens_map = match fetch_sensitivity(&net, &sta) {
        Ok(map) => map,
        Err(e) => {
            tracing::warn!("Could not fetch StationXML from FDSN: {}. Using default RS4D sensitivity (384,500).", e);
            let mut fallback = HashMap::new();
            fallback.insert("ENE".to_string(), 384500.0);
            fallback.insert("ENN".to_string(), 384500.0);
            fallback.insert("ENZ".to_string(), 384500.0);
            fallback.insert("EHZ".to_string(), 384500.0);
            fallback
        }
    };

    // 2. Setup Configs
    let trigger_config = TriggerConfig {
        sta_sec: 6.0,
        lta_sec: 30.0,
        threshold: 1.05,
        reset_threshold: 0.5,
        highpass: 0.1,
        lowpass: 2.0,
        target_channel: "EHZ".to_string(),
    };

    let target_channels: Vec<String> = args.channels.split(',').map(|s| s.to_string()).collect();
    let intensity_config = if target_channels.len() == 3 {
        let mut sensitivities = Vec::new();
        for ch in &target_channels {
            let s = sens_map.get(ch).cloned().unwrap_or(384500.0);
            sensitivities.push(1.0 / s);
        }
        Some(IntensityConfig {
            channels: target_channels,
            sample_rate: 100.0,
            sensitivities,
        })
    } else {
        None
    };

    // 4. Simulation or Live UDP mode
    let sm = sens_map.clone();
    if let Some(path) = args.file {
        tracing::info!("Simulation mode: processing file {}", path);
        let ws = web_state.clone();
        let pipeline_handle = tokio::spawn(async move {
            run_pipeline(pipe_rx, trigger_config, intensity_config, ws, sm).await;
        });

        let bytes = std::fs::read(&path).unwrap();
        for chunk in bytes.chunks(512) {
            let _ = pipe_tx.send(chunk.to_vec()).await;
        }
        
        drop(pipe_tx);
        let _ = pipeline_handle.await;
        tracing::info!("Simulation complete.");
        return;
    } else {
        // LIVE UDP MODE
        let ws = web_state.clone();
        tokio::spawn(async move {
            run_pipeline(pipe_rx, trigger_config, intensity_config, ws, sm).await;
        });

        let (recv_tx, mut recv_rx) = mpsc::channel(100);
        tokio::spawn(async move {
            if let Err(e) = start_receiver(args.udp_port, recv_tx).await {
                tracing::error!("Receiver error: {}", e);
            }
        });

        tokio::spawn(async move {
            while let Some(packet) = recv_rx.recv().await {
                let _ = pipe_tx.send(packet.data).await;
            }
        });
    }

    tracing::info!("Running in Live UDP mode. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await.unwrap();
}
