# Research: Playwright Alert Capture

**Branch**: `041-playwright-alert-capture` | **Date**: 2026-02-16

## R1: Playwright as a Persistent Screenshot Service

**Decision**: Use `playwright-core` (not full `playwright`) with the built-in Node.js `http` module as an HTTP screenshot server.

**Rationale**:
- `playwright-core` is the minimal package (no bundled browsers at install time) — browser is installed separately via `npx playwright install chromium`
- Playwright uses CDP (Chrome DevTools Protocol) directly — no ChromeDriver needed, eliminating version mismatch issues
- A persistent browser instance avoids the 2-5 second cold start per screenshot. A warm browser context takes ~200-500ms for page navigation + screenshot
- Built-in `http` module is sufficient for a single-endpoint service (no Express/Fastify overhead needed)

**Alternatives Considered**:
- **Puppeteer**: Similar capabilities but Playwright has better ARM64 support and multi-browser abstraction
- **Express + Playwright**: Unnecessary framework overhead for a single endpoint
- **Rust headless-chrome crate**: Limited ARM64 support, less maintained, harder to match exact WebUI rendering

## R2: ARM64 Linux (Raspberry Pi 4) Compatibility

**Decision**: Playwright officially supports ARM64 Linux since v1.30+ (2023-02). The bundled Chromium works on Raspberry Pi 4 with 64-bit OS.

**Rationale**:
- Playwright's `npx playwright install chromium` detects the architecture and downloads the correct ARM64 binary
- Headless Chromium on RPi4 (4GB RAM) typically uses 150-250MB resident memory
- With `MemoryMax=300M` in systemd, the service stays within budget even under load
- Page rendering + screenshot for a 1000×2000px canvas completes in ~5-15 seconds on RPi4

**Alternatives Considered**:
- **Chromium system package + playwright `channel: 'chromium'`**: Avoids bundled download but introduces version mismatch risk
- **Firefox via Playwright**: ARM64 Firefox support is less tested; Chromium is the primary target

## R3: Capture Page Design — Dedicated `/capture` Route

**Decision**: Add a dedicated Next.js page at `webui/src/app/capture/page.tsx` that renders the snapshot layout (4 channels vertically stacked, 1000×2000px), driven entirely by URL query parameters.

**Rationale**:
- Reuses the same `ChannelPairCanvas` component and rendering pipeline as the main WebUI page
- Query parameters control: time range, channels, filter settings, spectrogram settings, intensity badge
- The capture page fetches historical data from the rsudp-rust REST API (not live WebSocket), renders once, then signals readiness via a DOM marker (`data-capture-ready="true"`)
- This ensures pixel-identical rendering to the WebUI because the same Canvas 2D code draws both

**Key Query Parameters** (passed via URL to the capture page):
- `channels`: Comma-separated channel list (e.g., `EHZ,EHN,EHE,ENZ`)
- `start`: Start timestamp (ISO 8601)
- `end`: End timestamp (ISO 8601)
- `intensity_class`: JMA intensity class for badge (e.g., `3`, `5+`)
- `intensity_value`: Instrumental intensity value
- `backend_url`: rsudp-rust backend URL for data fetch (e.g., `http://localhost:8080`)
- `width`, `height`: Viewport dimensions (default 1000×2000)

**Note**: Display settings (bandpass filter, spectrogram frequency range, deconvolution) are NOT passed as URL query parameters. The capture page retrieves them from the `GET /api/capture/data` response (PlotSettings field), ensuring consistency with the current backend configuration.

**Alternatives Considered**:
- **Inject data via JavaScript**: More complex, harder to debug
- **Static HTML page outside Next.js**: Duplicates rendering code
- **Server-side rendering with Canvas (node-canvas)**: Different rendering engine, not pixel-identical

## R4: Data Flow for Capture

**Decision**: rsudp-rust exposes a new REST endpoint `GET /api/capture/data` that returns the waveform + spectrogram data for a given time range, suitable for the capture page to render.

**Rationale**:
- The existing `waveform_buffers` in `WebState` hold ~300 seconds of data per channel
- The existing spectrogram data is also buffered in `WebState`
- A dedicated endpoint avoids WebSocket complexity and provides a simple JSON response with all needed data
- The capture page fetches this data, renders the canvas, and signals readiness

**Data Response Format**:
```json
{
  "station": "AM.R6E01",
  "channels": {
    "EHZ": { "samples": [...], "start_time": "...", "sample_rate": 100.0 },
    "EHN": { "samples": [...], "start_time": "...", "sample_rate": 100.0 }
  },
  "spectrogram": {
    "EHZ": { "columns": [[...]], "frequency_bins": 64, "sample_rate": 100.0 },
    "EHN": { "columns": [[...]], "frequency_bins": 64, "sample_rate": 100.0 }
  },
  "intensity": { "max_class": "3", "max_value": 2.85 },
  "settings": {
    "filter_waveform": true,
    "filter_highpass": 0.7,
    "filter_lowpass": 9.0,
    "deconvolve": true,
    "spectrogram_freq_min": 0.0,
    "spectrogram_freq_max": 9.0
  }
}
```

**Alternatives Considered**:
- **Pass data as base64 in URL**: Too large for 4-channel waveform data
- **Write data to shared file**: Adds filesystem coupling between services

## R5: HTTP Communication Protocol (rsudp-rust → Capture Service)

**Decision**: rsudp-rust sends `POST /capture` to the capture service with a JSON body containing the capture parameters. The service returns the PNG binary directly.

**Rationale**:
- Simple request-response pattern fits the infrequent alert reset use case
- `reqwest` (already in Cargo.toml for other HTTP calls) handles the HTTP client side
- Response is raw PNG bytes (`Content-Type: image/png`), written directly to the alerts directory
- Timeout: 30 seconds (matching SC-001)

**Alternatives Considered**:
- **gRPC**: Overkill for a single endpoint; adds protobuf dependency to the Node.js service
- **Unix socket**: Faster but harder to debug and doesn't work across Docker containers
- **Message queue**: Unnecessary complexity for an infrequent, synchronous operation

## R6: Removal of plotters Dependency

**Decision**: Remove `draw_rsudp_plot()`, `generate_snapshot()`, and all plotters-related code from rsudp-rust. Replace with an HTTP call to the capture service.

**Rationale**:
- Per FR-005, the legacy plotters code must be removed
- `snapshot_path` in `NotificationEvent` becomes populated by the capture service response (or `None` if service unavailable)
- The `plotters` crate and related dependencies (`ab_glyph`, embedded fonts) can be removed from `Cargo.toml`, reducing binary size and compile time

**Files to Remove/Modify**:
- Remove: `src/web/plot.rs` (entire file)
- Modify: `src/web/alerts.rs` (replace `generate_snapshot` with HTTP capture call)
- Modify: `src/pipeline.rs` (update snapshot generation flow)
- Modify: `Cargo.toml` (remove plotters, ab_glyph, embedded font dependencies)
- Remove: `src/resources/` embedded font files (if only used by plotters)

## R7: systemd Service Configuration

**Decision**: Create `rsudp-capture.service` with `Nice=19`, `MemoryMax=300M`, running as the `rsudp` user.

**Rationale**:
- `Nice=19` ensures the capture service yields CPU to rsudp-rust (priority monitoring)
- `MemoryMax=300M` prevents the browser engine from consuming excessive memory on RPi4
- Same `rsudp` user as the main service; shared `/var/lib/rsudp` for output files
- `Restart=on-failure` with `RestartSec=10` for automatic recovery from browser crashes

**Service Dependencies**:
- `After=rsudp.service` — capture service starts after the monitoring service
- `Wants=rsudp.service` — but doesn't hard-depend (can run independently)

## R8: Installation via Makefile

**Decision**: Extend the existing `make install` target to include capture service installation.

**Rationale**:
- Add `install-capture` target that:
  1. Copies capture service source to `/opt/rsudp-capture/`
  2. Runs `npm install --production` in the capture service directory
  3. Runs `npx playwright install --with-deps chromium`
  4. Installs `rsudp-capture.service` to systemd
- The main `install` target calls `install-capture` as a dependency
- This satisfies FR-008 (single `make install` for everything)

**Alternatives Considered**:
- **Docker-only deployment**: Not suitable for bare-metal RPi4 installations
- **Snap/Flatpak packaging**: Too complex for this project's scope
