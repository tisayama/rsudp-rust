use crate::web::sns::{SNSProvider, NotificationEvent};
use async_trait::async_trait;
use reqwest::multipart;
use tokio::fs;

pub struct DiscordProvider {
    webhook_url: String,
    use_embed: bool,
}

impl DiscordProvider {
    pub fn new(webhook_url: String, use_embed: bool) -> Self {
        Self {
            webhook_url,
            use_embed,
        }
    }
}

#[async_trait]
impl SNSProvider for DiscordProvider {
    async fn send_trigger(&self, event: &NotificationEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let content = format!("🚨 **ALERT TRIGGERED** on channel `{}` at {} 🚨", event.channel, event.timestamp);
        
        let payload = if self.use_embed {
            serde_json::json!({
                "embeds": [{
                    "title": "Seismic Alert Triggered",
                    "description": content,
                    "color": 0xFF0000,
                    "timestamp": event.timestamp.to_rfc3339()
                }]
            })
        } else {
            serde_json::json!({ "content": content })
        };

        client.post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?;
            
        Ok(())
    }

    async fn send_reset(&self, event: &NotificationEvent, _image_url: Option<&str>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let content = format!(
            "✅ **ALERT RESET** on channel `{}`\nMax Ratio: {:.2}\nMax Intensity: {:.2}",
            event.channel, event.max_ratio, event.max_intensity
        );

        if let Some(path) = &event.snapshot_path {
            if path.exists() {
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let file_bytes = fs::read(path).await?;
                let part = multipart::Part::bytes(file_bytes)
                    .file_name(file_name.clone())
                    .mime_str("image/png")?;
                
                let form = multipart::Form::new()
                    .text("content", content)
                    .part("file", part);

                client.post(&self.webhook_url)
                    .multipart(form)
                    .send()
                    .await?;
                
                return Ok(())
            }
        }

        // Fallback to text-only if no image
        client.post(&self.webhook_url)
            .json(&serde_json::json!({ "content": content }))
            .send()
            .await?;

        Ok(())
    }
}
