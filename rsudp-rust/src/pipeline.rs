use tokio::sync::mpsc;
use tracing::{info, warn};
use std::collections::{HashMap, VecDeque};
use crate::parser::{parse_any};
use crate::trigger::{TriggerManager, TriggerConfig, AlertEventType};
use crate::intensity::{IntensityManager, IntensityConfig};
use crate::web::stream::WebState;
use crate::web::alerts::AlertEvent as WebAlertEvent;
use uuid::Uuid;
use std::time::Duration;
use chrono::Utc;

pub async fn run_pipeline(
    mut receiver: mpsc::Receiver<Vec<u8>>,
    trigger_config: TriggerConfig,
    intensity_config: Option<IntensityConfig>,
    web_state: WebState,
    sensitivity_map: HashMap<String, f64>,
) {
    info!("Pipeline started");
    let mut tm = TriggerManager::new(trigger_config);
    let mut im = intensity_config.map(IntensityManager::new);
    let mut active_alerts: HashMap<String, Uuid> = HashMap::new();
    let max_buffer_samples = (100.0 * 300.0) as usize; // 300 seconds at 100Hz

    while let Some(data) = receiver.recv().await {
        let segments = match parse_any(&data) {
            Ok(s) => s,
            Err(e) => {
                warn!("Parser error: {}", e);
                continue;
            }
        };

        for segment in segments {
            // 0. Update shared waveform buffers
            {
                let mut buffers = web_state.waveform_buffers.lock().unwrap();
                let buf = buffers.entry(segment.channel.clone()).or_insert_with(|| VecDeque::with_capacity(max_buffer_samples));
                for &sample in &segment.samples {
                    if buf.len() >= max_buffer_samples {
                        buf.pop_front();
                    }
                    buf.push_back(sample);
                }
            }

            // 1. STA/LTA Triggering
            let id = format!("{}.{}.{}.{}", segment.network, segment.station, segment.location, segment.channel);
            let sensitivity = sensitivity_map.get(&segment.channel).cloned().unwrap_or(384500.0);
            
            for &sample in &segment.samples {
                if let Some(alert) = tm.add_sample(&id, sample, segment.starttime, sensitivity) {
                    match alert.event_type {
                        AlertEventType::Trigger => {
                            let alert_id = Uuid::new_v4();
                            active_alerts.insert(id.clone(), alert_id);
                            
                            {
                                let mut max_ints = web_state.alert_max_intensities.lock().unwrap();
                                max_ints.insert(alert_id, -2.0);
                            }
                            
                            info!("{}", alert);
                            web_state.broadcast_alert_start(alert_id, segment.channel.clone(), alert.timestamp).await;
                            
                            let (settings, trigger_time) = {
                                let mut history = web_state.history.lock().unwrap();
                                history.add_event(WebAlertEvent {
                                    id: alert_id,
                                    channel: segment.channel.clone(),
                                    trigger_time: alert.timestamp,
                                    reset_time: None,
                                    max_ratio: alert.ratio,
                                    snapshot_path: None,
                                    message: None,
                                });
                                (history.get_settings(), alert.timestamp)
                            };
                            
                            // 1. Immediate Trigger Email
                            let ch = segment.channel.clone();
                            let t_settings_trig = settings.clone();
                            tokio::spawn(async move {
                                if let Err(e) = crate::web::alerts::send_trigger_email(&t_settings_trig, &ch, trigger_time) {
                                    warn!("Failed to send trigger email: {}", e);
                                }
                            });

                            // 2. Schedule Snapshot Generation & Summary Notification
                            let plot_settings = web_state.settings.read().unwrap().clone();
                            let delay = Duration::from_secs_f64(plot_settings.window_seconds * plot_settings.save_pct);
                            let shared_state = web_state.clone();
                            let alert_ch = segment.channel.clone();
                            let alert_sta = segment.station.clone();
                            let s_map = sensitivity_map.clone();
                            let t_settings_reset = settings.clone();

                            tokio::spawn(async move {
                                tokio::time::sleep(delay).await;
                                
                                let max_int = {
                                    let mut max_ints = shared_state.alert_max_intensities.lock().unwrap();
                                    max_ints.remove(&alert_id).unwrap_or(-2.0)
                                };
                                
                                let shindo_class = crate::intensity::get_shindo_class(max_int);
                                let intensity_message = crate::web::alerts::format_shindo_message(&shindo_class);
                                let sens_opt = s_map.get(&alert_ch).cloned();

                                let snapshot_samples = (100.0 * plot_settings.window_seconds) as usize;
                                let mut trimmed_data: HashMap<String, Vec<f64>> = HashMap::new();
                                
                                {
                                    let buffers = shared_state.waveform_buffers.lock().unwrap();
                                    for (c, b) in buffers.iter() {
                                        let start_idx = b.len().saturating_sub(snapshot_samples);
                                        let samples: Vec<f64> = b.range(start_idx..).cloned().collect();
                                        trimmed_data.insert(c.clone(), samples);
                                    }
                                }

                                // Generate snapshot
                                let snapshot_path = match crate::web::alerts::generate_snapshot(alert_id, &alert_sta, &trimmed_data, trigger_time, sens_opt, max_int) {
                                    Ok(path) => {
                                        let mut history = shared_state.history.lock().unwrap();
                                        history.set_snapshot_path(alert_id, path.clone());
                                        Some(path)
                                    },
                                    Err(e) => {
                                        warn!("Failed to generate snapshot: {}", e);
                                        None
                                    }
                                };

                                // Send Reset (Summary) Email
                                if let Err(e) = crate::web::alerts::send_reset_email(&t_settings_reset, &alert_ch, trigger_time, Utc::now(), max_int, snapshot_path.as_ref().map(|p| format!("http://localhost:8080/images/alerts/{}", p)).as_deref(), &intensity_message) {
                                    warn!("Failed to send reset email: {}", e);
                                }

                                // Update history and broadcast
                                {
                                    let mut history = shared_state.history.lock().unwrap();
                                    history.reset_event(alert_id, Utc::now(), max_int, intensity_message.clone());
                                }
                                shared_state.broadcast_alert_end(alert_id, alert_ch, Utc::now(), max_int, intensity_message).await;
                            });
                        },
                        AlertEventType::Reset => {
                            if active_alerts.remove(&id).is_some() {
                                info!("Alert state reset for {}", id);
                            }
                        }
                    }
                }
            }

            // 2. WebUI Waveform Stream
            web_state.broadcast_waveform(segment.channel.clone(), segment.starttime, segment.samples.clone()).await;

            // 3. Seismic Intensity Calculation
            if let Some(im) = im.as_mut() {
                let is_target = im.config().channels.iter().any(|target| id.contains(target));
                
                if is_target {
                    let mut map = HashMap::new();
                    let short_name = if id.contains("ENE") { "ENE" }
                                    else if id.contains("ENN") { "ENN" }
                                    else { "ENZ" };
                    
                    map.insert(short_name.to_string(), segment.samples.clone());
                    im.add_samples(map, segment.starttime);

                    for res in im.get_results() {
                        info!("[{}] 計測震度: {:.2} ({})", res.timestamp, res.intensity, res.shindo_class);
                        web_state.broadcast_intensity(res.clone()).await;
                        
                        {
                            let mut max_ints = web_state.alert_max_intensities.lock().unwrap();
                            for &alert_id in active_alerts.values() {
                                if let Some(peak) = max_ints.get_mut(&alert_id) {
                                    if res.intensity > *peak { *peak = res.intensity; }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
