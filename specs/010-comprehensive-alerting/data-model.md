# Data Model: Comprehensive Alerting System

## Entities

### 1. AlertEvent
Represents a historical or active seismic alert.

| Field | Type | Description |
|-------|------|-------------|
| id | UUID | Unique identifier for the alert session |
| channel | String | Channel that triggered the alert (e.g., EHZ) |
| trigger_time | DateTime | Start of the trigger event |
| reset_time | Option<DateTime> | End of the trigger event (if reset) |
| max_ratio | f64 | Highest STA/LTA ratio recorded |
| snapshot_path | Option<String> | Relative URL path to the generated PNG plot |

### 2. AlertSettings
Persistence for user-configurable alert behavior.

| Field | Type | Description |
|-------|------|-------------|
| audio_enabled | bool | Toggle for browser sound notifications |
| email_enabled | bool | Toggle for SMTP notifications |
| flash_enabled | bool | Toggle for visual UI flashing |
| smtp_host | String | SMTP server address |
| smtp_port | u16 | SMTP server port |
| smtp_user | String | SMTP username |
| smtp_pass | String | SMTP password (encrypted or env-masked) |
| email_recipient | String | Target email address |

## State Transitions

### Alert Lifecycle
1. **PENDING**: System starts up, LTA is warming up.
2. **MONITORING**: Averages are stable, waiting for trigger.
3. **TRIGGERED**: STA/LTA > Threshold. 
   - Broadcast "START" to WebUI.
   - Play Audio.
   - Send "Trigger" Email.
4. **RESET**: STA/LTA < Reset Threshold.
   - Broadcast "END" to WebUI.
   - Generate PNG Snapshot.
   - Send "Reset" Email with stats and image link.
   - Transition back to MONITORING.
