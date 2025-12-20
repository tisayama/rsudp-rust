use crate::intensity::{IntensityConfig, IntensityManager};
use crate::parser::parse_any;
use crate::trigger::{TriggerConfig, TriggerManager};
use crate::web::stream::WebState;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{info, warn};

pub async fn run_pipeline(
    mut receiver: mpsc::Receiver<Vec<u8>>,
    trigger_config: TriggerConfig,
    intensity_config: Option<IntensityConfig>,
    web_state: WebState,
) {
    info!("Pipeline started");
    let mut tm = TriggerManager::new(trigger_config);
    let mut im = intensity_config.map(IntensityManager::new);

    while let Some(data) = receiver.recv().await {
        let segments = match parse_any(&data) {
            Ok(s) => s,
            Err(e) => {
                warn!("Parser error: {}", e);
                continue;
            }
        };

        for segment in segments {
            // 1. STA/LTA Triggering
            let id = format!(
                "{}.{}.{}.{}",
                segment.network, segment.station, segment.location, segment.channel
            );
            for &sample in &segment.samples {
                if let Some(alert) = tm.add_sample(&id, sample, segment.starttime) {
                    info!("{}", alert);
                    web_state.broadcast_alert(alert).await;
                }
            }

            // 2. WebUI Waveform Stream
            web_state
                .broadcast_waveform(
                    segment.channel.clone(),
                    segment.starttime,
                    segment.samples.clone(),
                )
                .await;

            // 3. Seismic Intensity Calculation
            if let Some(im) = im.as_mut() {
                // Check if this channel is one of the 3-component targets
                let is_target = im
                    .config()
                    .channels
                    .iter()
                    .any(|target| id.contains(target));

                if is_target {
                    let mut map = HashMap::new();
                    // Map to simple channel name (ENE, ENN, ENZ) for internal manager
                    let short_name = if id.contains("ENE") {
                        "ENE"
                    } else if id.contains("ENN") {
                        "ENN"
                    } else {
                        "ENZ"
                    };

                    map.insert(short_name.to_string(), segment.samples.clone());
                    im.add_samples(map, segment.starttime);

                    for res in im.get_results() {
                        info!(
                            "[{}] 計測震度: {:.2} ({})",
                            res.timestamp, res.intensity, res.shindo_class
                        );
                        web_state.broadcast_intensity(res).await;
                    }
                }
            }
        }
    }
}
