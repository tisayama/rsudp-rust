use crate::web::sns::{SNSProvider, NotificationEvent};
use async_trait::async_trait;
use aws_sdk_sns::Client;

pub struct AwsSnsProvider {
    client: Client,
    topic_arn: String,
}

impl AwsSnsProvider {
    pub async fn new(topic_arn: String, region: String) -> Self {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region))
            .load()
            .await;
        let client = Client::new(&config);
        Self {
            client,
            topic_arn,
        }
    }
}

#[async_trait]
impl SNSProvider for AwsSnsProvider {
    async fn send_trigger(&self, event: &NotificationEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let message = format!("ALERT: {} triggered at {}", event.channel, event.timestamp);
        self.client.publish()
            .topic_arn(&self.topic_arn)
            .message(message)
            .send()
            .await?;
        Ok(())
    }

    async fn send_reset(&self, event: &NotificationEvent, _image_url: Option<&str>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let message = format!(
            "ALERT RESET: {}\nMax Ratio: {:.2}\nMax Intensity: {:.2}",
            event.channel, event.max_ratio, event.max_intensity
        );
        self.client.publish()
            .topic_arn(&self.topic_arn)
            .message(message)
            .send()
            .await?;
        Ok(())
    }
}
