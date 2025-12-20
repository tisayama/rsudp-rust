use crate::web::stream::WebState;
use chrono::Utc;

pub fn create_test_web_state() -> WebState {
    WebState::new()
}

// Dummy helper for tests if needed
pub async fn send_test_waveform(state: &WebState, channel: &str, samples: Vec<f64>) {
    state
        .broadcast_waveform(channel.to_string(), Utc::now(), samples)
        .await;
}
