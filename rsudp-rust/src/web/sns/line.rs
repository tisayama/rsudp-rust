use crate::web::sns::{SNSProvider, NotificationEvent};
use async_trait::async_trait;
use serde_json::json;
use tracing::error;

pub struct LineProvider {
    channel_access_token: String,
    to_ids: Vec<String>,
}

impl LineProvider {
    pub fn new(channel_access_token: String, to_ids: Vec<String>) -> Self {
        Self {
            channel_access_token,
            to_ids,
        }
    }

    async fn push_message(&self, message: serde_json::Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        for id in &self.to_ids {
            let payload = json!({
                "to": id,
                "messages": [message]
            });

            let res = client.post("https://api.line.me/v2/bot/message/push")
                .bearer_auth(&self.channel_access_token)
                .json(&payload)
                .send()
                .await?;

            if !res.status().is_success() {
                let err_body = res.text().await?;
                error!("LINE push failed for {}: {}", id, err_body);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl SNSProvider for LineProvider {
    async fn send_trigger(&self, event: &NotificationEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let text = format!("🚨 地震検知 🚨\nチャンネル: {}\n発生時刻: {}", event.channel, event.timestamp);
        self.push_message(json!({
            "type": "text",
            "text": text
        })).await
    }

    async fn send_reset(&self, event: &NotificationEvent, image_url: Option<&str>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let text = format!(
            "✅ 警報解除 ✅\nチャンネル: {}\n最大STA/LTA比: {:.2}\n最大震度: {:.2}",
            event.channel, event.max_ratio, event.max_intensity
        );
        
        // 1. Send text
        self.push_message(json!({
            "type": "text",
            "text": text
        })).await?;

        // 2. Send image if available
        if let Some(url) = image_url {
            self.push_message(json!({
                "type": "image",
                "originalContentUrl": url,
                "previewImageUrl": url
            })).await?;
        }

        Ok(())
    }
}
