use tokio::sync::mpsc;
use tracing::{info, warn};
use std::collections::{HashMap, VecDeque};
use crate::parser::{parse_any};
use crate::trigger::{TriggerManager, TriggerConfig, AlertEventType};
use crate::intensity::{IntensityManager, IntensityConfig};
use crate::web::stream::WebState;
use crate::web::alerts::AlertEvent as WebAlertEvent;
use uuid::Uuid;

pub async fn run_pipeline(
    mut receiver: mpsc::Receiver<Vec<u8>>,
    trigger_config: TriggerConfig,
    intensity_config: Option<IntensityConfig>,
    web_state: WebState,
) {
    info!("Pipeline started");
    let mut tm = TriggerManager::new(trigger_config);
    let mut im = intensity_config.map(IntensityManager::new);
    let mut active_alerts: HashMap<String, Uuid> = HashMap::new();
    let mut waveform_buffers: HashMap<String, VecDeque<f64>> = HashMap::new();
    let max_buffer_samples = (100.0 * 60.0) as usize; // 60 seconds at 100Hz

    while let Some(data) = receiver.recv().await {
        let segments = match parse_any(&data) {
            Ok(s) => s,
            Err(e) => {
                warn!("Parser error: {}", e);
                continue;
            }
        };

        for segment in segments {
            // 0. Update waveform buffers for alerts
            let buf = waveform_buffers.entry(segment.channel.clone()).or_insert_with(|| VecDeque::with_capacity(max_buffer_samples));
            for &sample in &segment.samples {
                if buf.len() >= max_buffer_samples {
                    buf.pop_front();
                }
                buf.push_back(sample);
            }

            // 1. STA/LTA Triggering
            let id = format!("{}.{}.{}.{}", segment.network, segment.station, segment.location, segment.channel);
            for &sample in &segment.samples {
                if let Some(alert) = tm.add_sample(&id, sample, segment.starttime) {
                    match alert.event_type {
                        AlertEventType::Trigger => {
                            let alert_id = Uuid::new_v4();
                            active_alerts.insert(id.clone(), alert_id);
                            
                            info!("{}", alert);
                            web_state.broadcast_alert_start(alert_id, segment.channel.clone(), alert.timestamp).await;
                            
                            let mut history = web_state.history.lock().unwrap();
                            let settings = history.get_settings();
                            history.add_event(WebAlertEvent {
                                id: alert_id,
                                channel: segment.channel.clone(),
                                trigger_time: alert.timestamp,
                                reset_time: None,
                                max_ratio: alert.ratio,
                                snapshot_path: None,
                            });
                            
                            // Send Trigger Email in background
                            let ch = segment.channel.clone();
                            let ts = alert.timestamp;
                            tokio::spawn(async move {
                                if let Err(e) = crate::web::alerts::send_trigger_email(&settings, &ch, ts) {
                                    warn!("Failed to send trigger email: {}", e);
                                }
                            });
                        },
                        AlertEventType::Reset => {
                            if let Some(alert_id) = active_alerts.remove(&id) {
                                info!("{}", alert);
                                
                                let (settings, trigger_time) = {
                                    let history = web_state.history.lock().unwrap();
                                    let trigger_time = history.get_events().iter().find(|e| e.id == alert_id).map(|e| e.trigger_time).unwrap_or(alert.timestamp);
                                    (history.get_settings(), trigger_time)
                                };

                                let ch = segment.channel.clone();
                                let ts = alert.timestamp;
                                let ratio = alert.ratio;
                                let samples: Vec<f64> = waveform_buffers.get(&ch).map(|b| b.iter().cloned().collect()).unwrap_or_default();
                                let shared_state = web_state.clone();

                                tokio::spawn(async move {
                                    // 1. Generate snapshot
                                    let snapshot_path = match crate::web::alerts::generate_snapshot(alert_id, &ch, &samples) {
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

                                    // 2. Send Reset Email
                                    let snapshot_url = snapshot_path.as_ref().map(|p| format!("http://localhost:8080/images/alerts/{}", p));
                                    if let Err(e) = crate::web::alerts::send_reset_email(&settings, &ch, trigger_time, ts, ratio, snapshot_url.as_deref()) {
                                        warn!("Failed to send reset email: {}", e);
                                    }
                                });

                                web_state.broadcast_alert_end(alert_id, segment.channel.clone(), alert.timestamp, alert.ratio).await;
                                
                                let mut history = web_state.history.lock().unwrap();
                                history.reset_event(alert_id, alert.timestamp, alert.ratio);
                            }
                        }
                    }
                }
            }

            // 2. WebUI Waveform Stream
            web_state.broadcast_waveform(segment.channel.clone(), segment.starttime, segment.samples.clone()).await;

            // 3. Seismic Intensity Calculation
            if let Some(im) = im.as_mut() {
                // Check if this channel is one of the 3-component targets
                let is_target = im.config().channels.iter().any(|target| id.contains(target));
                
                if is_target {
                    let mut map = HashMap::new();
                    // Map to simple channel name (ENE, ENN, ENZ) for internal manager
                    let short_name = if id.contains("ENE") { "ENE" }
                                    else if id.contains("ENN") { "ENN" }
                                    else { "ENZ" };
                    
                    map.insert(short_name.to_string(), segment.samples.clone());
                    im.add_samples(map, segment.starttime);

                    for res in im.get_results() {
                        info!("[{}] 計測震度: {:.2} ({})", res.timestamp, res.intensity, res.shindo_class);
                        web_state.broadcast_intensity(res).await;
                    }
                }
            }
        }
    }
}