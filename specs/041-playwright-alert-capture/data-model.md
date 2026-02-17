# Data Model: Playwright Alert Capture

**Branch**: `041-playwright-alert-capture` | **Date**: 2026-02-16

## Entities

### CaptureRequest

Sent from rsudp-rust to the capture service on alert reset.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `station` | string | yes | Station identifier (e.g., "AM.R6E01") |
| `channels` | string[] | yes | Channel list (e.g., ["EHZ", "EHN", "EHE", "ENZ"]) |
| `start_time` | string (ISO 8601) | yes | Waveform window start time |
| `end_time` | string (ISO 8601) | yes | Waveform window end time |
| `intensity_class` | string | yes | JMA seismic intensity class (e.g., "3", "5+") |
| `intensity_value` | number | yes | Instrumental intensity value |
| `backend_url` | string | yes | rsudp-rust backend URL for data fetch (e.g., "http://localhost:8080") |
| `width` | number | no | Output image width in pixels (default: 1000) |
| `height` | number | no | Output image height in pixels (default: 500 × N_channels) |

**Validation Rules**:
- `channels` must be non-empty (1-4 channels)
- `start_time` must be before `end_time`
- `intensity_class` must be a valid JMA intensity level: "0", "1", "2", "3", "4", "5-", "5+", "6-", "6+", "7"

### CaptureResponse

Returned by the capture service.

**Success** (HTTP 200):
- `Content-Type: image/png`
- Body: Raw PNG binary data

**Error** (HTTP 4xx/5xx):
- `Content-Type: application/json`

| Field | Type | Description |
|-------|------|-------------|
| `error` | string | Error message |
| `details` | string | Additional context (optional) |

### CaptureDataResponse

Returned by the rsudp-rust backend `GET /api/capture/data` endpoint for the capture page to consume.

| Field | Type | Description |
|-------|------|-------------|
| `station` | string | Station identifier |
| `sample_rate` | number | Sample rate in Hz |
| `channels` | object | Per-channel waveform data (see ChannelWaveform) |
| `spectrogram` | object | Per-channel spectrogram data (see ChannelSpectrogram) |
| `sensitivity` | object | Per-channel sensitivity values |
| `settings` | object | Current plot settings from config |

### ChannelWaveform

| Field | Type | Description |
|-------|------|-------------|
| `samples` | number[] | Waveform sample values (raw counts or physical units) |
| `start_time` | string (ISO 8601) | First sample timestamp |

### ChannelSpectrogram

| Field | Type | Description |
|-------|------|-------------|
| `columns` | number[][] | 2D array [time_column][frequency_bin], values 0-255 |
| `frequency_bins` | number | Number of frequency bins per column |
| `hop_duration` | number | Duration between spectrogram columns in seconds |
| `first_column_timestamp` | number | Unix timestamp (seconds) of first column |

## State Transitions

### Capture Service Lifecycle

```
STARTING → READY → PROCESSING → READY
                  ↘ ERROR → READY (auto-recovery)
```

- **STARTING**: Browser launching, initial page load
- **READY**: Browser warm, waiting for capture requests
- **PROCESSING**: Navigating to capture URL, waiting for render, taking screenshot
- **ERROR**: Browser crash or timeout; auto-restart via systemd

### Alert Notification Flow (Modified)

```
Alert Reset Event
  ↓
rsudp-rust: POST /capture to capture service
  ↓ (success)                    ↓ (failure/timeout)
Save PNG to alerts/              Log warning
  ↓                                ↓
Upload to S3 (if needed)         ─┐
  ↓                               │
Notify channels (with image)    Notify channels (without image)
```
