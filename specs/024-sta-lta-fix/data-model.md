# Data Model: STA/LTA Fix

## Entities

### `TriggerConfig` (Updated)
| Field | Type | Description |
|-------|------|-------------|
| duration | f64 | Seconds the ratio must exceed threshold before triggering |

### `StaLtaState` (Updated)
| Field | Type | Description |
|-------|------|-------------|
| warmup_complete | bool | False until LTA window is filled |
| samples_processed | u64 | Total samples seen |
| exceed_start | Option<DateTime<Utc>> | Timestamp when threshold was first exceeded |
| is_exceeding | bool | Current state of ratio > threshold |

## Logic

### Warm-up
`samples_needed = LTA_seconds * sample_rate`
If `samples_processed < samples_needed`:
  - Update STA/LTA/Filter
  - Return `None` (No trigger possible)

### Duration Timer
If `ratio > threshold`:
  - If `!is_exceeding`:
    - `is_exceeding = true`
    - `exceed_start = now`
  - If `now - exceed_start > duration`:
    - Fire TRIGGER (if not already triggered)
Else:
  - `is_exceeding = false`
  - `exceed_start = None`
  - If `ratio < reset`:
    - Fire RESET (if already triggered)
