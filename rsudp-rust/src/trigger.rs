use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertEventType {
    Trigger,
    Reset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub timestamp: DateTime<Utc>,
    pub channel: String,
    pub event_type: AlertEventType,
    pub ratio: f64,
    pub message: String,
}

impl std::fmt::Display for AlertEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] Channel {}: {}",
            self.timestamp, self.channel, self.message
        )
    }
}

#[derive(Debug, Clone)]
pub struct TriggerConfig {
    pub sta_sec: f64,
    pub lta_sec: f64,
    pub threshold: f64,
    pub reset_threshold: f64,
}

pub struct TriggerManager {
    config: TriggerConfig,
    states: HashMap<String, StaLtaState>,
}

struct StaLtaState {
    sta: f64,
    lta: f64,
    mean: f64,
    triggered: bool,
    max_ratio: f64,
}

impl TriggerManager {
    pub fn new(config: TriggerConfig) -> Self {
        Self {
            config,
            states: HashMap::new(),
        }
    }

    pub fn add_sample(
        &mut self,
        id: &str,
        sample: f64,
        timestamp: DateTime<Utc>,
    ) -> Option<AlertEvent> {
        let state = self.states.entry(id.to_string()).or_insert(StaLtaState {
            sta: 0.0,
            lta: 0.0,
            mean: sample,
            triggered: false,
            max_ratio: 0.0,
        });

        // 1. Update recursive mean (DC removal)
        let mean_alpha = 0.01;
        state.mean = (1.0 - mean_alpha) * state.mean + mean_alpha * sample;
        let val = (sample - state.mean).abs();

        // 2. Update STA/LTA
        let sta_alpha = 1.0 / (self.config.sta_sec * 100.0);
        let lta_alpha = 1.0 / (self.config.lta_sec * 100.0);

        state.sta = (1.0 - sta_alpha) * state.sta + sta_alpha * val;
        state.lta = (1.0 - lta_alpha) * state.lta + lta_alpha * val;

        let ratio = if state.lta > 0.0 {
            state.sta / state.lta
        } else {
            0.0
        };

        if !state.triggered && ratio > self.config.threshold {
            state.triggered = true;
            state.max_ratio = ratio;
            let message = format!(
                "Trigger threshold {} exceeded (ratio: {:.4}). ALARM!",
                self.config.threshold, ratio
            );
            return Some(AlertEvent {
                timestamp,
                channel: id.to_string(),
                event_type: AlertEventType::Trigger,
                ratio,
                message,
            });
        } else if state.triggered {
            state.max_ratio = state.max_ratio.max(ratio);
            if ratio < self.config.reset_threshold {
                state.triggered = false;
                let message = format!(
                    "Ratio {:.4} fell below reset threshold {}. RESET. Max ratio: {:.4}",
                    ratio, self.config.reset_threshold, state.max_ratio
                );
                let ev = AlertEvent {
                    timestamp,
                    channel: id.to_string(),
                    event_type: AlertEventType::Reset,
                    ratio,
                    message,
                };
                return Some(ev);
            }
        }

        None
    }
}