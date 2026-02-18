use rsudp_rust::pipeline::run_pipeline;
use rsudp_rust::sound::AudioManager;
use rsudp_rust::hue::HueIntegration;
use rsudp_rust::trigger::TriggerConfig;
use rsudp_rust::intensity::IntensityConfig;
use rsudp_rust::web::WebState;
use rsudp_rust::receiver::start_receiver;
use rsudp_rust::parser::stationxml::{fetch_sensitivity, fetch_response};
use rsudp_rust::parser::mseed::parse_mseed_file;
use rsudp_rust::settings::Settings;
use tokio::sync::mpsc;
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

use rsudp_rust::forward::ForwardManager;
use rsudp_rust::pubsub;
use rsudp_rust::rsam::RsamManager;
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
        let mut default_settings = Settings::default();
        
        // Manually populate intensity keys for dump output
        // We do this here instead of in Default implementation to avoid
        // config crate parsing errors (Char error) when loading defaults
        let mut intensity_files = std::collections::BTreeMap::new();
        for key in ["0", "1", "2", "3", "4", "5-", "5+", "6-", "6+", "7"] {
            intensity_files.insert(key.to_string(), "".to_string());
        }
        default_settings.alertsound.intensity_files = intensity_files;

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
    
    // Initialize Hue Integration
    let hue_integration = HueIntegration::new(settings.hue.clone());
    hue_integration.start().await; // Starts discovery loop

    // Initialize Audio Manager (Optional, if device exists)
    let audio_manager = if settings.alertsound.enabled {
        match AudioManager::new() {
            Some(am) => Some(Arc::new(am)),
            None => {
                tracing::warn!("Audio playback enabled but no output device found. Sound will be disabled.");
                None
            }
        }
    } else {
        None
    };

    tracing::info!("LOADED CONFIG: threshold={}, reset={}, port={}", settings.alert.threshold, settings.alert.reset, settings.settings.port);

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
        plot_settings.deconvolve = settings.plot.deconvolve;
        plot_settings.units = settings.plot.units.clone();
        plot_settings.show_spectrogram = settings.plot.spectrogram;
        plot_settings.spectrogram_freq_min = settings.plot.lower_limit;
        plot_settings.spectrogram_freq_max = settings.plot.upper_limit;
        plot_settings.spectrogram_log_y = settings.plot.logarithmic_y_axis;
        plot_settings.filter_waveform = settings.plot.filter_waveform;
        plot_settings.filter_highpass = settings.plot.filter_highpass;
        plot_settings.filter_lowpass = settings.plot.filter_lowpass;
        plot_settings.filter_corners = settings.plot.filter_corners as usize;
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
    let web_port = args.web_port.unwrap_or(8080); // Default to 8080 if not specified
    let addr = format!("0.0.0.0:{}", web_port);
    let app_state = web_state.clone();
    tokio::spawn(async move {
        let router = rsudp_rust::web::routes::create_router(app_state).await;
        let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind WebUI port - is rsudp already running?");
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
    {
        let mut sn = web_state.station_name.write().unwrap();
        *sn = sta.clone();
    }
    // Fetch full instrument response (poles/zeros) for frequency-domain deconvolution
    let resp_map = match fetch_response(&net, &sta).await {
        Ok(map) => {
            tracing::info!("Fetched instrument response for {} channels", map.len());
            map
        }
        Err(e) => {
            tracing::warn!("Could not fetch instrument response from FDSN: {}. Using fallback sensitivities.", e);
            HashMap::new()
        }
    };

    // Build sensitivity map from response data (or use fallback)
    let sens_map = if !resp_map.is_empty() {
        resp_map.iter().map(|(k, v)| (k.clone(), v.sensitivity)).collect()
    } else {
        match fetch_sensitivity(&net, &sta).await {
            Ok(map) => map,
            Err(e) => {
                tracing::warn!("Could not fetch StationXML from FDSN: {}. Using default Raspberry Shake sensitivities.", e);
                let mut fallback = HashMap::new();
                fallback.insert("ENE".to_string(), 384500.0);
                fallback.insert("ENN".to_string(), 384500.0);
                fallback.insert("ENZ".to_string(), 384500.0);
                fallback.insert("EHZ".to_string(), 399000000.0);
                fallback
            }
        }
    };

    // Populate maps in WebState for WebUI deconvolution
    {
        let mut sm = web_state.sensitivity_map.write().unwrap();
        *sm = sens_map.clone();
    }
    {
        let mut rm = web_state.response_map.write().unwrap();
        *rm = resp_map;
    }

    // 2. Setup Configs
    let trigger_config = TriggerConfig {
        sta_sec: settings.alert.sta,
        lta_sec: settings.alert.lta,
        threshold: settings.alert.threshold,
        reset_threshold: settings.alert.reset,
        highpass: settings.alert.highpass,
        lowpass: settings.alert.lowpass,
        target_channel: settings.alert.channel.clone(),
        duration: settings.alert.duration,
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

    // 5. Initialize Forward Manager
    let forward_manager = if settings.forward.enabled {
        match ForwardManager::new(&settings.forward).await {
            Ok(fm) => Some(Arc::new(fm)),
            Err(e) => {
                tracing::error!("Forward configuration error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    // 6. Initialize RSAM Manager
    let rsam_manager = if settings.rsam.enabled {
        match RsamManager::new(&settings.rsam, sens_map.clone()) {
            Ok(rm) => Some(rm),
            Err(e) => {
                tracing::error!("RSAM configuration error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    // 7. Initialize Pub/Sub Client (if enabled)
    let pubsub_client = if settings.pubsub.enabled {
        match pubsub::create_pubsub_client(&settings.pubsub).await {
            Ok(client) => {
                tracing::info!("Pub/Sub client initialized successfully");
                Some(client)
            }
            Err(e) => {
                tracing::warn!("Pub/Sub initialization failed: {}. Continuing without Pub/Sub.", e);
                None
            }
        }
    } else {
        None
    };

    // 8. Simulation or Live UDP mode
    let sm = sens_map.clone();
    let sns = Some(sns_manager.clone());
    let hue = Some(hue_integration.clone());
    let audio = audio_manager.clone();
    let sound_settings = settings.alertsound.clone();
    if let Some(path) = args.file {
        tracing::info!("Simulation mode: processing file {}", path);
        let ws = web_state.clone();
        let sns_sim = sns.clone();
        let hue_sim = hue.clone();
        let audio_sim = audio.clone();
        let sound_sim = sound_settings.clone();
        let fwd_sim = forward_manager.clone();
        let rsam_sim = rsam_manager;
        let pipeline_handle = tokio::spawn(async move {
            run_pipeline(pipe_rx, trigger_config, intensity_config, ws, sm, sns_sim, hue_sim, audio_sim, sound_sim, fwd_sim, rsam_sim, None, settings.capture.clone()).await;
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
        // LIVE MODE
        let ws = web_state.clone();
        let sns_live = sns.clone();
        let hue_live = hue.clone();
        let audio_live = audio.clone();
        let sound_live = sound_settings.clone();
        let fwd_live = forward_manager.clone();
        let rsam_live = rsam_manager;

        // Determine publisher sender (for UDP input mode with Pub/Sub enabled)
        let publisher_tx = if settings.pubsub.enabled && settings.pubsub.input_mode == "udp" {
            if let Some(ref client) = pubsub_client {
                match pubsub::publisher::start_publisher(
                    client,
                    &settings.pubsub,
                    &format!("{}.{}", net, sta),
                ).await {
                    Ok(tx) => {
                        tracing::info!("Pub/Sub publisher started");
                        Some(tx)
                    }
                    Err(e) => {
                        tracing::warn!("Failed to start Pub/Sub publisher: {}. Continuing without publishing.", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        tokio::spawn(async move {
            run_pipeline(pipe_rx, trigger_config, intensity_config, ws, sm, sns_live, hue_live, audio_live, sound_live, fwd_live, rsam_live, publisher_tx, settings.capture.clone()).await;
        });

        if settings.pubsub.enabled && settings.pubsub.input_mode == "pubsub" {
            // SUBSCRIBER MODE: receive data from Pub/Sub instead of UDP
            if let Some(ref client) = pubsub_client {
                let cancel = tokio_util::sync::CancellationToken::new();
                match pubsub::subscriber::start_subscriber(
                    client,
                    &settings.pubsub,
                    &format!("{}.{}", net, sta),
                    pipe_tx,
                    cancel,
                ).await {
                    Ok(()) => tracing::info!("Pub/Sub subscriber started"),
                    Err(e) => {
                        tracing::error!("Failed to start Pub/Sub subscriber: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                tracing::error!("Pub/Sub input_mode is 'pubsub' but client is not available");
                std::process::exit(1);
            }
        } else {
            // UDP MODE (default)
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
    }

    tracing::info!("Running in Live UDP mode. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await.unwrap();
}