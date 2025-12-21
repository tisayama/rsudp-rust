use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use chrono::{Duration, Utc};
use crate::web::alerts::{AlertEvent, AlertSettings};

pub struct AlertHistoryManager {
    events: VecDeque<AlertEvent>,
    settings: AlertSettings,
}

impl AlertHistoryManager {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            settings: AlertSettings::default(),
        }
    }

    pub fn add_event(&mut self, event: AlertEvent) {
        self.events.push_back(event);
        self.cleanup();
    }

    pub fn update_event(&mut self, event: AlertEvent) {
        if let Some(pos) = self.events.iter().position(|e| e.id == event.id) {
            self.events[pos] = event;
        }
    }

    pub fn reset_event(&mut self, id: uuid::Uuid, reset_time: chrono::DateTime<Utc>, max_ratio: f64) {
        if let Some(event) = self.events.iter_mut().find(|e| e.id == id) {
            event.reset_time = Some(reset_time);
            event.max_ratio = max_ratio;
        }
    }

    pub fn set_snapshot_path(&mut self, id: uuid::Uuid, path: String) {
        if let Some(event) = self.events.iter_mut().find(|e| e.id == id) {
            event.snapshot_path = Some(path);
        }
    }

    pub fn get_events(&self) -> Vec<AlertEvent> {
        self.events.iter().cloned().collect()
    }

    pub fn get_settings(&self) -> AlertSettings {
        self.settings.clone()
    }

    pub fn update_settings(&mut self, settings: AlertSettings) {
        self.settings = settings;
    }

    fn cleanup(&mut self) {
        let cutoff = Utc::now() - Duration::hours(24);
        while let Some(event) = self.events.front() {
            if event.trigger_time < cutoff {
                // Delete image file if exists
                if let Some(ref path) = event.snapshot_path {
                    let full_path = format!("alerts/{}", path);
                    if std::path::Path::new(&full_path).exists() {
                        let _ = std::fs::remove_file(full_path);
                    }
                }
                self.events.pop_front();
            } else {
                break;
            }
        }
    }
}

pub type SharedHistory = Arc<Mutex<AlertHistoryManager>>;
