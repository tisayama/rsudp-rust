use std::path::PathBuf;
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use crate::trigger::AlertEventType;
use crate::settings::Settings;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct NotificationEvent {
    pub event_type: AlertEventType,
    pub timestamp: DateTime<Utc>,
    pub station_id: String,
    pub channel: String,
    pub max_ratio: f64,
    pub max_intensity: f64,
    pub snapshot_path: Option<PathBuf>,
}

#[async_trait]
pub trait SNSProvider: Send + Sync {
    async fn send_trigger(&self, event: &NotificationEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn send_reset(&self, event: &NotificationEvent, image_url: Option<&str>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub mod s3;
pub mod discord;
pub mod line;
pub mod gchat;
pub mod aws_sns;

use self::s3::S3Client;
use self::discord::DiscordProvider;
use self::line::LineProvider;
use self::gchat::GChatProvider;
use self::aws_sns::AwsSnsProvider;

pub struct SNSManager {
    providers: Vec<Arc<dyn SNSProvider>>,
    s3_client: Option<Arc<S3Client>>,
}

impl SNSManager {
    pub async fn from_settings(settings: &Settings) -> Self {
        let mut manager = Self {
            providers: Vec::new(),
            s3_client: None,
        };

        // Initialize S3 if needed (for LINE or GChat)
        if (settings.line.enabled && settings.line.send_images) || 
           (settings.googlechat.enabled && settings.googlechat.send_images) {
            if let (Some(bucket), Some(region)) = (settings.googlechat.s3_bucket_name.clone().or(settings.line.s3_bucket_name.clone()),
                                                   Some(settings.googlechat.s3_aws_region.clone().unwrap_or_else(|| settings.line.s3_aws_region.clone().unwrap_or_else(|| "us-east-1".to_string())))) {
                manager.s3_client = Some(Arc::new(S3Client::new(bucket, region).await));
            }
        }

        // Discord
        if settings.discord.enabled {
            manager.providers.push(Arc::new(DiscordProvider::new(
                settings.discord.webhook_url.clone(),
                settings.discord.use_embed
            )));
        }

        // LINE
        if settings.line.enabled {
            manager.providers.push(Arc::new(LineProvider::new(
                settings.line.channel_access_token.clone(),
                settings.line.to_ids.split(',').map(|s| s.trim().to_string()).collect()
            )));
        }

        // Google Chat
        if settings.googlechat.enabled {
            manager.providers.push(Arc::new(GChatProvider::new(
                settings.googlechat.webhook_url.clone()
            )));
        }

        // Amazon SNS
        if settings.sns.enabled {
            manager.providers.push(Arc::new(AwsSnsProvider::new(
                settings.sns.topic_arn.clone(),
                settings.sns.aws_region.clone()
            ).await));
        }

        manager
    }

    pub async fn notify_trigger(&self, event: &NotificationEvent) {
        for provider in &self.providers {
            let provider = provider.clone();
            let event = event.clone();
            tokio::spawn(async move {
                if let Err(e) = provider.send_trigger(&event).await {
                    tracing::warn!("Failed to send trigger notification: {}", e);
                }
            });
        }
    }

    pub async fn notify_reset(&self, event: &NotificationEvent) {
        let mut image_url = None;

        // Upload to S3 if we have a client and a snapshot
        if let (Some(s3), Some(path)) = (&self.s3_client, &event.snapshot_path) {
            if path.exists() {
                let key = format!("alerts/{}.png", uuid::Uuid::new_v4());
                match s3.upload_image(path, &key).await {
                    Ok(url) => image_url = Some(url),
                    Err(e) => tracing::warn!("Failed to upload image to S3: {}", e),
                }
            }
        }

        let image_url_shared = image_url.map(Arc::new);

        for provider in &self.providers {
            let provider = provider.clone();
            let event = event.clone();
            let img_url = image_url_shared.clone();
            tokio::spawn(async move {
                let url_str = img_url.as_ref().map(|u| u.as_str());
                if let Err(e) = provider.send_reset(&event, url_str).await {
                    tracing::warn!("Failed to send reset notification: {}", e);
                }
            });
        }
    }
}
