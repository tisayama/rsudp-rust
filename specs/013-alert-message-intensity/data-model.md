# Data Model: Intensity Inclusion in Alert Messages

## Entities

### 1. WebAlertEvent (Updated)
Updated to include the descriptive message field.

| Field | Type | Description |
|-------|------|-------------|
| id | Uuid | Unique alert ID |
| channel | String | Channel name |
| trigger_time | DateTime | Start time |
| reset_time | Option<DateTime> | End time |
| max_ratio | f64 | Peak STA/LTA |
| snapshot_path | Option<String> | PNG path |
| **message** | **String** | Descriptive Japanese text based on intensity |

## Message Generation Logic

The `message` field is populated during the `Reset` event:

```rust
fn format_shindo_message(shindo_class: &str) -> String {
    if shindo_class == "0" {
        "揺れを検出できませんでした".to_string()
    } else {
        let display_class = match shindo_class {
            "5-" => "5弱",
            "5+" => "5強",
            "6-" => "6弱",
            "6+" => "6強",
            _ => shindo_class,
        };
        format!("震度 {}相当の揺れを検出しました", display_class)
    }
}
```
