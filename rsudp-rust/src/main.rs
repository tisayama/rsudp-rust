use rsudp_rust::pipeline::run_pipeline;
use rsudp_rust::trigger::TriggerConfig;
use rsudp_rust::intensity::IntensityConfig;
use rsudp_rust::web::WebState;
use rsudp_rust::receiver::start_receiver;
use rsudp_rust::parser::stationxml::fetch_sensitivity;
use rsudp_rust::parser::mseed::parse_mseed_file;
use rsudp_rust::settings::Settings;
use tokio::sync::mpsc;
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

use rsudp_rust::web::sns::SNSManager;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// MiniSEED file for simulation
    #[arg(short, long)]
    file: Option<String>,

    /// Path to configuration file (TOML/YAML)
    #[arg(short = 'C', long)]
    config: Option<PathBuf>,

    /// Dump default configuration to file and exit
    #[arg(long)]
    dump_config: Option<PathBuf>,

    /// Network code (overrides config)
    #[arg(short, long)]
    network: Option<String>,

    /// Station name (overrides config)
    #[arg(short, long)]
    station: Option<String>,

    /// Channels for intensity calculation (must be 3, overrides config)
    #[arg(short, long)]
    channels: Option<String>,

    /// WebUI port (overrides config)
    #[arg(short, long)]
    web_port: Option<u16>,

    /// UDP port for receiver (overrides config)
    #[arg(short, long)]
    udp_port: Option<u16>,

    /// Seconds of data to display in plot (overrides config)
    #[arg(long)]
    window_seconds: Option<f64>,
    
    /// Directory to save plots and data (overrides config)
    #[arg(short, long)]
    output_dir: Option<PathBuf>,

    /// Ratio of duration to wait before posting (overrides config)
    #[arg(long)]
    save_pct: Option<f64>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    // 1. Handle Config Dumping
    if let Some(dump_path) = args.dump_config {
        let default_settings = Settings::default();
        let format = dump_path.extension().and_then(|e| e.to_str()).unwrap_or("toml");
        match default_settings.dump(format) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&dump_path, content) {
                    eprintln!("Error writing config: {}", e);
                    std::process::exit(1);
                }
                println!("Default configuration dumped to {:?}", dump_path);
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Error generating config: {}", e);
                std::process::exit(1);
            }
        }
    }

    // 2. Load Settings
    let mut settings = match Settings::new(args.config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            std::process::exit(1);
        }
    };

    // 3. Override Settings with CLI Args
    if let Some(p) = args.udp_port { settings.settings.port = p; }
    if let Some(s) = args.station { settings.settings.station = s; }
    if let Some(o) = args.output_dir { settings.settings.output_dir = o; }
    // Note: window_seconds and save_pct are merged below into web_state and config

    // 4. Initialize WebState with merged settings
    let web_state = WebState::new();
    {
        let mut plot_settings = web_state.settings.write().unwrap();
        // Use plot.duration from config as default for window_seconds
        plot_settings.window_seconds = args.window_seconds.unwrap_or(settings.plot.duration as f64);
        
        // Use a default save_pct since it's not in the main config yet, or use arg
        if let Some(sp) = args.save_pct {
            plot_settings.save_pct = sp;
        }
        
        plot_settings.output_dir = settings.settings.output_dir.clone();
    }
    
    // Update default history settings as well
    {
        let mut history = web_state.history.lock().unwrap();
        let mut h_settings = history.get_settings();
        h_settings.save_pct = args.save_pct.unwrap_or(0.7);
        history.update_settings(h_settings);
    }

    let (pipe_tx, pipe_rx) = mpsc::channel(100);

    // 1. Start Web Server
    let web_port = args.web_port.unwrap_or(8081); // Default to 8081 if not specified
    let addr = format!("0.0.0.0:{}", web_port);
    let app_state = web_state.clone();
    tokio::spawn(async move {
        let router = rsudp_rust::web::routes::create_router(app_state).await;
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        tracing::info!("WebUI server listening on {}", addr);
        axum::serve(listener, router).await.unwrap();
    });

    // 2. Determine target station and fetch metadata
    let net = args.network.unwrap_or_else(|| "AM".to_string());
    let (net, sta) = if let Some(path) = &args.file {
        // Peek at file to find station
        match parse_mseed_file(path) {
            Ok(segs) if !segs.is_empty() => (segs[0].network.clone(), segs[0].station.clone()),
            _ => (net, settings.settings.station.clone()),
        }
    } else {
        (net, settings.settings.station.clone())
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
        sta_sec: settings.alert.sta,
        lta_sec: settings.alert.lta,
        threshold: settings.alert.threshold,
        reset_threshold: settings.alert.reset,
        highpass: settings.alert.highpass,
        lowpass: settings.alert.lowpass,
        target_channel: settings.alert.channel.clone(),
    };

    let channels_str = args.channels.unwrap_or_else(|| "ENE,ENN,ENZ".to_string());
    let target_channels: Vec<String> = channels_str.split(',').map(|s| s.to_string()).collect();
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

    // 4. Initialize SNS Manager
    let sns_manager = Arc::new(SNSManager::from_settings(&settings).await);

    // 5. Simulation or Live UDP mode
    let sm = sens_map.clone();
    let sns = Some(sns_manager.clone());
    if let Some(path) = args.file {
        tracing::info!("Simulation mode: processing file {}", path);
        let ws = web_state.clone();
        let sns_sim = sns.clone();
        let pipeline_handle = tokio::spawn(async move {
            run_pipeline(pipe_rx, trigger_config, intensity_config, ws, sm, sns_sim).await;
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
        let sns_live = sns.clone();
        tokio::spawn(async move {
            run_pipeline(pipe_rx, trigger_config, intensity_config, ws, sm, sns_live).await;
        });

        let (recv_tx, mut recv_rx) = mpsc::channel(100);
        let udp_port = settings.settings.port;
        tokio::spawn(async move {
            if let Err(e) = start_receiver(udp_port, recv_tx).await {
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