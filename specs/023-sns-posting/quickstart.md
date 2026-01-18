# Quickstart: Configuring SNS Notifications

## Prerequisites
- AWS Account (S3 bucket and SNS Topic)
- Discord Webhook URL
- LINE Messaging API Channel Token and User/Group ID

## Configuration

Update your `settings.toml` or `settings.yaml`:

```toml
[discord]
enabled = true
webhook_url = "https://discord.com/api/webhooks/..."

[line]
enabled = true
channel_access_token = "YOUR_TOKEN"
to_ids = "USER_ID_1,GROUP_ID_2"
send_images = true

[sns]
enabled = true
topic_arn = "arn:aws:sns:us-east-1:123456789012:MyTopic"
aws_region = "us-east-1"

[googlechat]
enabled = true
webhook_url = "https://chat.googleapis.com/v1/spaces/..."
```

## Running the Application

1. Build with new dependencies: `cargo build`
2. Run normally: `./target/debug/rsudp-rust`
3. Trigger an alert using the `streamer` tool to verify delivery.
