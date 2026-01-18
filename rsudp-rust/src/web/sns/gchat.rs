use crate::web::sns::{SNSProvider, NotificationEvent};
use async_trait::async_trait;
use serde_json::json;

pub struct GChatProvider {
    webhook_url: String,
}

impl GChatProvider {
    pub fn new(webhook_url: String) -> Self {
        Self { webhook_url }
    }
}

#[async_trait]
impl SNSProvider for GChatProvider {
    async fn send_trigger(&self, event: &NotificationEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let text = format!("🚨 *ALERT TRIGGERED* on channel `{}` at {} 🚨", event.channel, event.timestamp);
        
        client.post(&self.webhook_url)
            .json(&json!({ "text": text }))
            .send()
            .await?;
            
        Ok(())
    }

    async fn send_reset(&self, event: &NotificationEvent, image_url: Option<&str>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let header = format!("✅ *ALERT RESET* on channel `{}`", event.channel);
        let content = format!("Max Ratio: {:.2}\nMax Intensity: {:.2}", event.max_ratio, event.max_intensity);

        let payload = if let Some(url) = image_url {
            json!({
                "cards": [{
                    "header": { "title": header },
                    "sections": [{
                        "widgets": [
                            { "textParagraph": { "text": content } },
                            { "image": { "imageUrl": url } }
                        ]
                    }]
                }]
            })
        } else {
            json!({ "text": format!("{}\n{}", header, content) })
        };

        client.post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?;

        Ok(())
    }
}
