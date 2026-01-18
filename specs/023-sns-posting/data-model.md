# Data Model: SNS Posting

## Entities

### `SNSManager`
Orchestrates notification delivery across multiple providers.

| Field | Type | Description |
|-------|------|-------------|
| clients | Vec<Arc<dyn SNSProvider>> | List of enabled provider implementations |

### `NotificationEvent`
Internal data structure for alerts.

| Field | Type | Description |
|-------|------|-------------|
| event_type | Enum | Trigger or Reset |
| timestamp | DateTime<Utc> | Event occurrence time |
| station_id | String | e.g. "AM.R6E01" |
| channel | String | e.g. "EHZ" |
| max_intensity | f64 | Calculated JMA intensity |
| snapshot_path | Option<PathBuf> | Local path to PNG plot |

### `S3Metadata` (for LINE/Google Chat)
Results from S3 upload.

| Field | Type | Description |
|-------|------|-------------|
| public_url | String | URL accessible by SNS providers |

## Provider Interfaces

### `SNSProvider` (Trait)
- `fn send_trigger(&self, event: &NotificationEvent) -> Future`
- `fn send_reset(&self, event: &NotificationEvent, image_url: Option<&str>) -> Future`
