use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HueConfig {
    pub enabled: bool,
    pub app_key: String,
    pub bridge_id: Option<String>,
    pub target_ids: Vec<String>,
}

impl Default for HueConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            app_key: "".to_string(),
            bridge_id: None,
            target_ids: Vec::new(),
        }
    }
}
