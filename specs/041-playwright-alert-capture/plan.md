# Implementation Plan: Playwright Alert Capture

**Branch**: `041-playwright-alert-capture` | **Date**: 2026-02-16 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/041-playwright-alert-capture/spec.md`

## Summary

Replace the current plotters-based alert plot generation in rsudp-rust with a Playwright headless browser screenshot approach. A standalone Node.js capture service holds a persistent Chromium instance and, upon HTTP request from rsudp-rust, navigates to a dedicated WebUI capture page, takes a screenshot, and returns the PNG. This ensures generated alert images are pixel-identical to the live WebUI. The legacy plotters code is removed entirely.

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2021) for backend changes; Node.js 18+ / TypeScript for capture service; TypeScript / Next.js 14 for WebUI capture page
**Primary Dependencies**: `playwright-core` (Node.js, headless browser), `reqwest` (Rust, HTTP client — already present), Next.js 14 (WebUI — existing)
**Storage**: Local filesystem (`/var/lib/rsudp/alerts/`) for PNG output
**Testing**: `cargo test` (Rust unit/integration), `jest` (WebUI unit), manual E2E (capture service + rsudp-rust)
**Target Platform**: x86_64 Linux, ARM64 Linux (Raspberry Pi 4)
**Project Type**: Web application (backend + frontend + standalone service)
**Performance Goals**: Screenshot generation < 30 seconds (including RPi4)
**Constraints**: Capture service memory < 300MB, no interference with rsudp-rust UDP reception
**Scale/Scope**: Infrequent captures (few per day), single station, 4 channels

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. 安定性と信頼性 | ✅ PASS | Capture service failure is isolated; rsudp-rust continues without image. systemd auto-restart for recovery. |
| II. 厳密なテスト | ✅ PASS | Unit tests for data API, capture service health check, integration test for capture flow. Manual E2E on RPi4. |
| III. 高いパフォーマンス | ✅ PASS | 30s timeout acceptable for alert reset (infrequent). Nice=19 + MemoryMax=300M prevents resource contention. |
| IV. コードの明瞭性 | ✅ PASS | Clean separation: capture service (Node.js), WebUI page, backend API. Removes dual rendering code (plotters). |
| V. 日本語仕様策定 | ✅ PASS | Spec written and clarified. |
| VI. 標準技術スタック | ✅ PASS | WebUI uses Next.js + Tailwind CSS (existing). Capture service uses Node.js (per assumption). |
| VII. 自己検証の義務 | ✅ PASS | E2E verification planned before commit. |
| VIII. ブランチ運用 | ✅ PASS | Feature branch `041-playwright-alert-capture`. |

**Post-Phase 1 Re-check**: All gates still pass. The addition of a Node.js service is justified by the requirement to reuse the existing Canvas 2D rendering code.

## Project Structure

### Documentation (this feature)

```text
specs/041-playwright-alert-capture/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 research findings
├── data-model.md        # Entity definitions and state transitions
├── quickstart.md        # Development setup guide
├── contracts/
│   ├── capture-service-api.yaml  # Capture service OpenAPI spec
│   └── capture-data-api.yaml     # Backend data endpoint spec
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Task breakdown (created by /speckit.tasks)
```

### Source Code (repository root)

```text
# Rust backend (existing, modified)
rsudp-rust/src/
├── web/
│   ├── plot.rs          # REMOVE — plotters rendering code
│   ├── alerts.rs        # MODIFY — replace generate_snapshot() with HTTP capture call
│   ├── routes.rs        # MODIFY — add GET /api/capture/data endpoint
│   └── sns/mod.rs       # MODIFY — handle missing snapshot gracefully
├── pipeline.rs          # MODIFY — update snapshot generation flow
├── settings.rs          # MODIFY — add [capture] config section
└── Cargo.toml           # MODIFY — remove plotters deps, keep reqwest

# Capture service (new)
capture-service/
├── server.js            # HTTP server + Playwright screenshot logic
├── package.json         # playwright-core dependency
└── package-lock.json

# WebUI (existing, modified)
webui/src/app/
└── capture/
    └── page.tsx         # NEW — dedicated capture rendering page

# systemd (new service file)
rsudp-rust/systemd/
├── rsudp.service            # EXISTING — no change
└── rsudp-capture.service    # NEW — capture service unit file

# Makefile (modified)
Makefile                     # MODIFY — add install-capture target
```

**Structure Decision**: This is a multi-component web application. The capture service is a new standalone Node.js project (`capture-service/`) placed at the repository root alongside the existing `rsudp-rust/` and `webui/` directories. This follows the existing pattern of co-located services.

## Detailed Design

### Component 1: Capture Service (`capture-service/`)

A minimal Node.js HTTP server using `playwright-core`:

1. **Startup**: Launch headless Chromium with `playwright-core`, keep browser instance alive
2. **`POST /capture`**:
   - Parse CaptureRequest JSON body
   - Build URL: `{webui_url}/capture?channels=EHZ,EHN,...&start=...&end=...&intensity=3&...`
   - Navigate a browser page to the URL
   - Wait for `[data-capture-ready="true"]` attribute on `<body>` (max 25s)
   - Take element screenshot of the capture container (1000 × 500*N px)
   - Return PNG bytes with `Content-Type: image/png`
3. **`GET /health`**: Return JSON with service status, browser connectivity, uptime
4. **Error handling**: If browser crashes, auto-reconnect. If page timeout, return 500.
5. **Concurrency**: Process one capture at a time (sequential queue). Discard if queue > 3.

**Port**: 9100 (configurable via `CAPTURE_PORT` env var)

### Component 2: WebUI Capture Page (`webui/src/app/capture/page.tsx`)

A dedicated Next.js page that:

1. Reads capture parameters from URL query string (channels, start, end, intensity_class, intensity_value, backend_url, width, height)
2. Fetches waveform + spectrogram data and display settings (PlotSettings) from `GET /api/capture/data?channels=...&start=...&end=...`
3. Applies display settings (bandpass filter, spectrogram frequency range, deconvolution) from the data API response — not from URL query parameters
4. Renders using the existing `ChannelPairCanvas` component (same rendering code as live page)
5. Renders the `IntensityBadge` component with the provided intensity class (all 10 JMA classes: 0, 1, 2, 3, 4, 5-, 5+, 6-, 6+, 7)
6. Sets `document.body.dataset.captureReady = "true"` when all canvases finish rendering
7. Uses a fixed viewport size (no responsive layout) — 1000px wide, 500px per channel pair

**Layout**: Vertical stack of channel pairs (Z, E, N, ENZ), matching the 1000 × (500 * N_channels) target dimensions.

### Component 3: Backend Data API (`rsudp-rust/src/web/routes.rs`)

New endpoint `GET /api/capture/data`:

1. Parse query parameters: `channels`, `start`, `end`
2. Extract waveform samples from `WebState::waveform_buffers` for the time range
3. Extract spectrogram columns from `WebState::spectrogram_data` for the time range
4. Include sensitivity values and current plot settings
5. Return JSON response (see `CaptureDataResponse` in data-model.md)

### Component 4: Rust Alert Flow Modification

**`pipeline.rs`** — Replace the plotters snapshot path:

```
Current:
  AlertReset → wait(save_pct * window) → generate_snapshot() → notify_reset(with image)

New:
  AlertReset → wait(save_pct * window) → POST /capture to capture service
    → if OK: save PNG to alerts/, notify_reset(with image)
    → if FAIL: log warning, notify_reset(without image)
```

**`alerts.rs`** — Replace `generate_snapshot()`:
- New function `capture_screenshot(capture_url, request)` using `reqwest::Client`
- Timeout: 30 seconds
- On success: write PNG to `{output_dir}/alerts/{uuid}.png`, return path
- On failure: return `None`, log warning

**`settings.rs`** — Add `[capture]` section:
```rust
pub struct CaptureSettings {
    pub enabled: bool,          // default: false
    pub service_url: String,    // default: "http://localhost:9100"
    pub timeout_seconds: u64,   // default: 30
}
```

### Component 5: systemd Service

`rsudp-capture.service`:
```ini
[Unit]
Description=rsudp Playwright Capture Service
After=network.target rsudp.service

[Service]
Type=simple
User=rsudp
Group=rsudp
WorkingDirectory=/opt/rsudp-capture
ExecStart=/usr/bin/node /opt/rsudp-capture/server.js
Nice=19
MemoryMax=300M
Restart=on-failure
RestartSec=10
Environment=CAPTURE_PORT=9100
Environment=WEBUI_URL=http://localhost:3000

[Install]
WantedBy=multi-user.target
```

### Component 6: Makefile Extension

Add `install-capture` target:
```makefile
install-capture:
    install -d /opt/rsudp-capture
    cp capture-service/server.js capture-service/package.json /opt/rsudp-capture/
    cd /opt/rsudp-capture && npm install --production
    cd /opt/rsudp-capture && npx playwright install --with-deps chromium
    install -m 644 rsudp-rust/systemd/rsudp-capture.service $(SYSTEMD_DIR)/
    systemctl daemon-reload
```

### Component 7: plotters Removal

Remove from `Cargo.toml`:
- `plotters` crate
- `ab_glyph` (if only used by plot.rs)
- Embedded font resources (`NotoSansJP-Bold.otf`, `DejaVuSansCondensed.ttf`)

Remove files:
- `rsudp-rust/src/web/plot.rs`
- Font files in `rsudp-rust/src/resources/` (if only used by plotters)

Update `rsudp-rust/src/web/mod.rs` to remove `mod plot;` declaration.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| New Node.js service (capture-service/) | Must reuse WebUI Canvas 2D rendering code, which runs in a browser | Rust-only rendering (plotters) produces visually different output from WebUI |
| Cross-service HTTP communication | Capture service must be process-isolated (FR-006) for resource control | In-process browser embedding would tie Chromium lifecycle to rsudp-rust |
