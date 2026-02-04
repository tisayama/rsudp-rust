use tokio::sync::mpsc;
use tracing::{info, warn};
use std::collections::HashMap;
use crate::parser::{parse_any};
use crate::trigger::{TriggerManager, TriggerConfig, AlertEventType};
use crate::intensity::{IntensityManager, IntensityConfig};
use crate::web::stream::{WebState, ChannelBuffer};
use crate::web::alerts::AlertEvent as WebAlertEvent;
use uuid::Uuid;
use std::time::{Duration, Instant};
use chrono::Utc;

use crate::web::sns::{SNSManager, NotificationEvent};
use std::sync::Arc;

pub async fn run_pipeline(
    mut receiver: mpsc::Receiver<Vec<u8>>,
    trigger_config: TriggerConfig,
    intensity_config: Option<IntensityConfig>,
    web_state: WebState,
    sensitivity_map: HashMap<String, f64>,
    sns_manager: Option<Arc<SNSManager>>,
) {
    info!("Pipeline started");
    let mut tm = TriggerManager::new(trigger_config);
    let mut im = intensity_config.map(IntensityManager::new);
    let mut active_alerts: HashMap<String, Uuid> = HashMap::new();
    let max_buffer_samples = (100.0 * 300.0) as usize; 

    // --- LOGGING STATE ---
    let mut last_log_time = Instant::now();
    let mut max_ratio_window: f64 = 0.0;
    let mut max_intensity_window: f64 = -9.9;

    while let Some(data) = receiver.recv().await {
        let segments = match parse_any(&data) {
            Ok(s) => s,
            Err(e) => {
                warn!("Parser error: {}", e);
                continue;
            }
        };

        for segment in segments {
            {
                let mut buffers = web_state.waveform_buffers.lock().unwrap();
                let buf = buffers.entry(segment.channel.clone())
                    .or_insert_with(|| ChannelBuffer::new(max_buffer_samples, segment.sampling_rate));
                
                buf.push_segment(segment.starttime, &segment.samples, max_buffer_samples);
            }

            let id = format!("{}.{}.{}.{}", segment.network, segment.station, segment.location, segment.channel);
            let sensitivity = 1.0; 
            
            // --- TRIGGER ---
            for (i, &sample) in segment.samples.iter().enumerate() {
                let sample_ts = segment.starttime + chrono::Duration::nanoseconds((i as f64 * 1_000_000_000.0 / segment.sampling_rate) as i64);
                
                if let Some(alert) = tm.add_sample(&id, sample, sample_ts, sensitivity) {
                    match alert.event_type {
                        AlertEventType::Trigger => {
                            let alert_id = Uuid::new_v4();
                            active_alerts.insert(id.clone(), alert_id);
                            info!("Triggered: {}. Acquiring max_ints lock...", id);
                            {
                                let mut max_ints = web_state.alert_max_intensities.lock().unwrap();
                                max_ints.insert(alert_id, -2.0);
                            }
                            info!("{}", alert);
                            
                            web_state.broadcast_alert_start(alert_id, segment.channel.clone(), alert.timestamp).await;
                            let (settings, trigger_time) = {
                                let mut history = web_state.history.lock().unwrap();
                                history.add_event(WebAlertEvent {
                                    id: alert_id, channel: segment.channel.clone(), trigger_time: alert.timestamp, reset_time: None, max_ratio: alert.ratio, snapshot_path: None, message: None,
                                });
                                (history.get_settings(), alert.timestamp)
                            };
                            let ch = segment.channel.clone();
                            let t_settings_trig = settings.clone();
                            tokio::spawn(async move {
                                if let Err(e) = crate::web::alerts::send_trigger_email(&t_settings_trig, &ch, trigger_time) { warn!("Failed to send trigger email: {}", e); }
                            });
                            if let Some(sns) = sns_manager.clone() {
                                let event = NotificationEvent {
                                    event_type: AlertEventType::Trigger, timestamp: alert.timestamp, station_id: format!("{}.{}", segment.network, segment.station), channel: segment.channel.clone(), max_ratio: alert.ratio, max_intensity: 0.0, snapshot_path: None,
                                };
                                tokio::spawn(async move { sns.notify_trigger(&event).await; });
                            }
                            
                            let plot_settings = web_state.settings.read().unwrap().clone();
                            let delay = Duration::from_secs_f64(plot_settings.window_seconds * plot_settings.save_pct);
                            let shared_state = web_state.clone();
                            let alert_ch = segment.channel.clone();
                            let alert_sta = segment.station.clone();
                            let s_map = sensitivity_map.clone();
                            let t_settings_reset = settings.clone();
                            let sns_for_reset = sns_manager.clone();

                            tokio::spawn(async move {
                                tokio::time::sleep(delay).await;
                                let max_int = {
                                    let mut max_ints = shared_state.alert_max_intensities.lock().unwrap();
                                    // Don't remove here if pipeline reset needs it? 
                                    // Actually pipeline reset happens much later or earlier.
                                    // Pipeline resets when ratio drops. Snapshot task runs independently.
                                    // We should peek or clone, but remove is safer to clean up memory.
                                    // However, pipeline needs it for RESET log.
                                    // Let's assume snapshot task finishes AFTER reset usually? No, reset depends on ratio.
                                    // If reset happens first, pipeline removes it. Then snapshot task gets -2.0.
                                    // If snapshot happens first, snapshot removes it. Then pipeline gets -9.9.
                                    // Solution: Do not remove in snapshot task if we want pipeline logging.
                                    // But pipeline logging is critical.
                                    // Let's rely on pipeline to remove it on RESET.
                                    // Snapshot task can just get current value.
                                    match max_ints.get(&alert_id) {
                                        Some(&v) => v,
                                        None => -2.0
                                    }
                                };
                                let shindo_class = crate::intensity::get_shindo_class(max_int);
                                let intensity_message = crate::web::alerts::format_shindo_message(&shindo_class);
                                let sens_opt = s_map.get(&alert_ch).cloned();
                                let pre_trigger_duration = plot_settings.window_seconds * (1.0 - plot_settings.save_pct);
                                let target_start_time = trigger_time - chrono::Duration::milliseconds((pre_trigger_duration * 1000.0) as i64);
                                let (trimmed_data, actual_start_time) = {
                                    let buffers = shared_state.waveform_buffers.lock().unwrap();
                                    let mut data = HashMap::new();
                                    let mut common_start_time = target_start_time;
                                    for (c, b) in buffers.iter() {
                                        let (samples, st) = b.extract_window(target_start_time, plot_settings.window_seconds);
                                        data.insert(c.clone(), samples);
                                        if c == &alert_ch { common_start_time = st; }
                                    }
                                    (data, common_start_time)
                                };
                                let out_dir = plot_settings.output_dir.clone();
                                let snapshot_path = if plot_settings.eq_screenshots {
                                    match crate::web::alerts::generate_snapshot(alert_id, &alert_sta, &trimmed_data, actual_start_time, sens_opt, max_int, &out_dir) {
                                        Ok(path) => {
                                            let mut history = shared_state.history.lock().unwrap();
                                            history.set_snapshot_path(alert_id, path.clone());
                                            Some(path)
                                        },
                                        Err(e) => { warn!("Failed to generate snapshot: {}", e); None }
                                    }
                                } else {
                                    None
                                };
                                if let Err(e) = crate::web::alerts::send_reset_email(&t_settings_reset, &alert_ch, trigger_time, Utc::now(), max_int, snapshot_path.as_ref().map(|p| format!("http://localhost:8080/images/alerts/{}", p)).as_deref(), &intensity_message) { warn!("Failed to send reset email: {}", e); }
                                if let Some(sns) = sns_for_reset.clone() {
                                    let event = NotificationEvent {
                                        event_type: AlertEventType::Reset, timestamp: Utc::now(), station_id: alert_sta.clone(), channel: alert_ch.clone(), max_ratio: alert.max_ratio, max_intensity: max_int, snapshot_path: snapshot_path.as_ref().map(|p| out_dir.join("alerts").join(p)),
                                    };
                                    tokio::spawn(async move { sns.notify_reset(&event).await; });
                                }
                                {
                                    let mut history = shared_state.history.lock().unwrap();
                                    history.reset_event(alert_id, Utc::now(), max_int, intensity_message.clone());
                                }
                                shared_state.broadcast_alert_end(alert_id, alert_ch, Utc::now(), max_int, intensity_message).await;
                            });
                        },
                        AlertEventType::Reset => {
                            if let Some(alert_id) = active_alerts.remove(&id) {
                                let max_int = {
                                    let mut max_ints = web_state.alert_max_intensities.lock().unwrap();
                                    // Remove here to clean up
                                    max_ints.remove(&alert_id).unwrap_or(-9.9)
                                };
                                let shindo = crate::intensity::get_shindo_class(max_int);
                                info!("{} | Max Intensity: {:.2} (JMA: {})", alert, max_int, shindo);
                            }
                        },
                        AlertEventType::Status => {
                            max_ratio_window = max_ratio_window.max(alert.ratio);
                        }
                    }
                }
            }

            web_state.broadcast_waveform(segment.channel.clone(), segment.starttime, segment.samples.clone()).await;

            // --- INTENSITY ---
            if let Some(im) = im.as_mut() {
                let is_target = im.config().channels.iter().any(|target| id.contains(target));
                if is_target {
                    let mut map = HashMap::new();
                    let short_name = if id.contains("ENE") { "ENE" } else if id.contains("ENN") { "ENN" } else { "ENZ" };
                    map.insert(short_name.to_string(), segment.samples.clone());
                    im.add_samples(map, segment.starttime);
                    for res in im.get_results() {
                        max_intensity_window = max_intensity_window.max(res.intensity);
                        
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
        
        // --- PERIODIC LOGGING ---
        if last_log_time.elapsed() >= Duration::from_secs(60) {
            info!("Status [60s]: Max STA/LTA={:.2}, Max Intensity={:.2}", max_ratio_window, max_intensity_window);
            max_ratio_window = 0.0;
            max_intensity_window = -9.9;
            last_log_time = Instant::now();
        }
    }
}
