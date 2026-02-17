# Quickstart: Playwright Alert Capture

**Branch**: `041-playwright-alert-capture` | **Date**: 2026-02-16

## Prerequisites

- Node.js v18+ installed
- Rust toolchain (cargo, rustc)
- rsudp-rust built and running (or streamer for simulation)

## Development Setup

### 1. Install Capture Service Dependencies

```bash
cd capture-service
npm install
npx playwright install --with-deps chromium
```

### 2. Start rsudp-rust Backend

```bash
cd rsudp-rust
cargo run -- --config ../rsudp.toml/rsudp.toml
```

### 3. Start WebUI (includes capture page)

```bash
cd webui
npm run dev
```

### 4. Start Capture Service

```bash
cd capture-service
node server.js
# Listens on http://localhost:9100
```

### 5. Test a Capture

```bash
# Manual test via curl
curl -X POST http://localhost:9100/capture \
  -H 'Content-Type: application/json' \
  -d '{
    "station": "AM.R6E01",
    "channels": ["EHZ", "EHN", "EHE", "ENZ"],
    "start_time": "2026-02-16T10:00:00Z",
    "end_time": "2026-02-16T10:01:30Z",
    "intensity_class": "3",
    "intensity_value": 2.85,
    "backend_url": "http://localhost:8080"
  }' --output test-capture.png

# Open the output
xdg-open test-capture.png
```

### 6. Health Check

```bash
curl http://localhost:9100/health
# {"status":"ready","browser_connected":true,"uptime_seconds":42,"captures_completed":0}
```

## Production Installation

```bash
make install
# Installs rsudp-rust, webui, and capture service

systemctl start rsudp
systemctl start rsudp-capture

# Verify
systemctl status rsudp-capture
curl http://localhost:9100/health
```

## Configuration

The capture service port is configured in `rsudp.toml`:

```toml
[capture]
enabled = true
service_url = "http://localhost:9100"
timeout_seconds = 30
```

## Architecture Diagram

```
                    ┌──────────────┐
                    │  rsudp-rust  │
                    │  (port 8080) │
                    └──────┬───────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
              ▼            ▼            ▼
        ┌──────────┐ ┌──────────┐ ┌───────────────┐
        │ UDP recv │ │ WebSocket│ │ POST /capture  │
        │ pipeline │ │ (live UI)│ │ (on alert     │
        │          │ │          │ │  reset)        │
        └──────────┘ └──────────┘ └───────┬───────┘
                                          │
                                          ▼
                                 ┌────────────────┐
                                 │ Capture Service │
                                 │ (port 9100)     │
                                 │ Node.js +       │
                                 │ Playwright      │
                                 └────────┬────────┘
                                          │
                              ┌───────────┼───────────┐
                              │           │           │
                              ▼           ▼           ▼
                        GET /api/   Navigate to   Screenshot
                        capture/    /capture?...  → PNG
                        data
                              │
                              ▼
                        ┌──────────┐
                        │  WebUI   │
                        │ /capture │
                        │  page    │
                        └──────────┘
```
