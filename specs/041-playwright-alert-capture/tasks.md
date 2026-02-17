# Tasks: Playwright Alert Capture

**Input**: Design documents from `/specs/041-playwright-alert-capture/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required per Constitution Principle II (厳密なテスト). Unit tests for new Rust endpoints and functions, health check test for capture service, integration test for capture page rendering.

**Organization**: Tasks grouped by user story. US1 is the MVP — the complete capture pipeline. US2 covers installation/operations. US3 covers non-interference verification.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Exact file paths included in descriptions

---

## Phase 1: Setup

**Purpose**: Create the new capture-service project and establish shared configuration

- [x] T001 Create capture-service/ directory at repository root with package.json containing `playwright-core` dependency and `node server.js` start script
- [x] T002 [P] Add CaptureSettings struct (enabled: bool, service_url: String, timeout_seconds: u64) with defaults to rsudp-rust/src/settings.rs and wire into Settings deserialization

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Backend types and endpoint that the capture page and capture service both depend on

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 Add CaptureDataResponse, ChannelWaveform, ChannelSpectrogram, and PlotSettings serializable structs to rsudp-rust/src/web/routes.rs per contracts/capture-data-api.yaml
- [x] T004 Implement GET /api/capture/data endpoint in rsudp-rust/src/web/routes.rs: parse channels/start/end query params, extract waveform samples and spectrogram columns from WebState buffers for the time range, return CaptureDataResponse JSON. Return 404 with error JSON when no data exists for the specified time range per capture-data-api.yaml 404 response schema

**Checkpoint**: Backend data API ready — capture page and capture service development can begin

---

## Phase 3: User Story 1 — High-Quality Plot Image Generation on Alert Reset (Priority: P1) MVP

**Goal**: On alert reset, generate a PNG plot image via Playwright headless browser that is visually identical to the WebUI, and deliver it through existing notification channels

**Independent Test**: Start capture service + WebUI + rsudp-rust, trigger alert reset, verify PNG is generated and delivered to notification channels

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T005 [P] [US1] Unit test for GET /api/capture/data endpoint in rsudp-rust/src/web/routes.rs #[cfg(test)]: test valid channel/time query returns CaptureDataResponse JSON, test missing required params returns 400, test empty time range with no buffered data returns 404
- [x] T006 [P] [US1] Unit test for capture_screenshot() function in rsudp-rust/src/web/alerts.rs #[cfg(test)]: mock HTTP server returning PNG bytes verifies file written to alerts/ directory and returns Some(path), mock HTTP timeout verifies returns None with warning log
- [x] T007 [P] [US1] Test capture-service health check and POST /capture in capture-service/: start server.js, verify GET /health returns {"status":"ready","browser_connected":true}, verify POST /capture with invalid JSON body returns 400 error response (manual validation — requires Playwright + Chromium installed per quickstart.md)
- [x] T008 [P] [US1] Integration test for capture page rendering in webui/: verify /capture page with test data renders ChannelPairCanvas for 4 channels, verify IntensityBadge renders correctly for all 10 JMA intensity classes (0, 1, 2, 3, 4, 5-, 5+, 6-, 6+, 7), verify data-capture-ready attribute is set on body element after rendering completes (manual validation — requires WebUI dev server per quickstart.md)

### Implementation for User Story 1

- [x] T009 [P] [US1] Create capture-service/server.js: HTTP server on port 9100 (CAPTURE_PORT env), persistent Chromium browser via playwright-core, POST /capture endpoint (parse CaptureRequest JSON, build WebUI capture URL, navigate, wait for data-capture-ready attribute, screenshot, return PNG), GET /health endpoint, sequential capture queue that rejects new requests with HTTP 503 when queue depth exceeds 3, graceful shutdown on SIGTERM/SIGINT per contracts/capture-service-api.yaml
- [x] T010 [P] [US1] Create WebUI capture page at webui/src/app/capture/page.tsx: read capture parameters from URL query string (channels, start, end, intensity_class, intensity_value, backend_url), fetch data including PlotSettings from GET /api/capture/data, render using existing ChannelPairCanvas component from webui/components/ChannelPairCanvas.tsx, render IntensityBadge component with provided intensity_class (all 10 JMA classes: 0, 1, 2, 3, 4, 5-, 5+, 6-, 6+, 7), set document.body.dataset.captureReady="true" when all canvases finish, fixed layout 1000px wide x 500px per channel pair
- [x] T011 [US1] Implement capture_screenshot() async function in rsudp-rust/src/web/alerts.rs: build CaptureRequest JSON from alert context (station, channels, time range, intensity, backend_url), POST to capture service URL from CaptureSettings using reqwest::Client with 30s timeout, on success write PNG to {output_dir}/alerts/{uuid}.png and return Some(path), on failure log warning and return None
- [x] T012 [US1] Update alert reset flow in rsudp-rust/src/pipeline.rs: replace generate_snapshot() call with capture_screenshot() call, pass CaptureSettings from config, handle None result (continue without image)
- [x] T013 [US1] Update rsudp-rust/src/web/sns/mod.rs: ensure notify_reset and all notification providers (Discord, LINE, Google Chat, SNS) handle snapshot_path being None gracefully — send notification text without image attachment when no screenshot is available
- [x] T014 [US1] Remove plotters rendering code: delete rsudp-rust/src/web/plot.rs, remove `pub mod plot;` from rsudp-rust/src/web/mod.rs
- [x] T015 [US1] Remove plotters dependencies from rsudp-rust/Cargo.toml: remove `plotters` crate (line 27), remove embedded font files rsudp-rust/src/resources/DejaVuSansCondensed.ttf, rsudp-rust/src/resources/NotoSansJP-Bold.otf, rsudp-rust/src/resources/NotoSansJP-Bold.ttf

**Checkpoint**: Full capture pipeline functional — alert reset generates PNG via Playwright and delivers through notifications. All US1 tests pass.

---

## Phase 4: User Story 2 — Capture Service Installation and Operation (Priority: P2)

**Goal**: Single `make install` installs all components including capture service and browser engine, managed via systemd

**Independent Test**: Run `make install` on a clean system, verify `systemctl start rsudp-capture` starts the service and `curl http://localhost:9100/health` returns ready status

### Implementation for User Story 2

- [x] T016 [P] [US2] Create rsudp-rust/systemd/rsudp-capture.service: Type=simple, User=rsudp, Group=rsudp, WorkingDirectory=/opt/rsudp-capture, ExecStart=/usr/bin/node server.js, Nice=19, MemoryMax=300M, Restart=on-failure, RestartSec=10, Environment CAPTURE_PORT=9100 and WEBUI_URL=http://localhost:3000, After=network.target rsudp.service
- [x] T017 [P] [US2] Add install-capture target to Makefile: install -d /opt/rsudp-capture, copy server.js and package.json, npm install --production, npx playwright install --with-deps chromium, install rsudp-capture.service to $(SYSTEMD_DIR), systemctl daemon-reload
- [x] T018 [US2] Update main `install` target in Makefile to call install-capture as a dependency

**Checkpoint**: `make install` installs everything, systemd manages capture service lifecycle

---

## Phase 5: User Story 3 — Non-Interference with Monitoring Process (Priority: P3)

**Goal**: Capture service operation does not impact rsudp-rust UDP reception, STA/LTA calculation, or other critical monitoring functions

**Independent Test**: On RPi4, trigger capture during continuous data streaming and verify no UDP packet loss or STA/LTA interruptions

**Note**: Most non-interference guarantees are implemented through US1 (graceful failure handling) and US2 (systemd resource limits). This phase focuses on hardening and verification.

### Implementation for User Story 3

- [x] T019 [US3] Verify and harden capture_screenshot() error handling in rsudp-rust/src/web/alerts.rs: ensure reqwest timeout is exactly 30s, connection errors and HTTP 5xx responses return None without panic, all failure paths log at warn level with error context, no blocking of the pipeline tokio task
- [x] T020 [US3] Verify rsudp-rust/systemd/rsudp-capture.service resource constraints: confirm Nice=19 (lowest CPU priority), MemoryMax=300M (hard memory ceiling), Restart=on-failure with RestartSec=10 (auto-recovery from browser crashes)

**Checkpoint**: Process isolation verified — capture service failures do not affect monitoring

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Build verification, cleanup, and end-to-end validation

- [x] T021 Run `cargo build --release --manifest-path rsudp-rust/Cargo.toml` and verify compilation succeeds without plotters dependencies
- [x] T022 Run `cargo test --manifest-path rsudp-rust/Cargo.toml` and fix any broken tests from plotters removal or alerts.rs refactor
- [x] T023 Run quickstart.md validation: start rsudp-rust backend, WebUI, and capture service, execute manual capture via curl per quickstart.md test command, verify PNG output. Verify on ARM64 if available, or document ARM64 as manual verification step.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on T002 (CaptureSettings) — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 completion — Tests (T005-T008) first, then T009, T010 can start in parallel
- **US2 (Phase 4)**: Depends on T009 (capture-service/server.js exists) — can run in parallel with late US1 tasks
- **US3 (Phase 5)**: Depends on US1 and US2 completion — verification phase
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Depends on Foundational (Phase 2). This is the MVP — delivers the complete capture pipeline
- **US2 (P2)**: Depends on T009 (server.js must exist before install target). Can be worked on in parallel with T011-T015
- **US3 (P3)**: Depends on US1 + US2 completion. Verification-focused — confirms non-interference properties

### Within User Story 1

```
T005-T008 (tests, write first) ────────────── parallel group (fail initially)
                                               ↓
T009 (capture service) ────────────────────────┐
T010 (capture page) ──────────────────────────┤ parallel group
T003/T004 (data API) ────────────────────────┘
                                               ↓
T011 (capture_screenshot in alerts.rs) ─────── depends on T009 contract
                                               ↓
T012 (pipeline.rs update) ─────────────────── depends on T011
                                               ↓
T013 (sns/mod.rs update) ─────────────────── depends on T012
T014 (remove plot.rs) ────────────────────── parallel with T013
T015 (remove Cargo deps + fonts) ──────────── depends on T014
```

### Parallel Opportunities

- **Phase 1**: T001 and T002 can run in parallel (different projects)
- **Phase 2**: T003 and T004 are sequential (same file)
- **Phase 3 Tests**: T005, T006, T007, T008 can all run in parallel (different projects/files)
- **Phase 3 Impl**: T009, T010 can run in parallel with each other (capture-service/ vs webui/)
- **Phase 4**: T016, T017 can run in parallel (systemd/ vs Makefile)
- **Phase 5**: T019, T020 can run in parallel (different files)

---

## Parallel Example: User Story 1

```bash
# Write all tests first (parallel — different projects):
Task: T005 "Unit test for GET /api/capture/data"
Task: T006 "Unit test for capture_screenshot()"
Task: T007 "Test capture-service health/POST"
Task: T008 "Integration test for capture page rendering"

# Launch capture service AND capture page in parallel (different projects):
Task: T009 "Create capture-service/server.js"
Task: T010 "Create webui/src/app/capture/page.tsx"

# After both complete, implement Rust-side integration sequentially:
Task: T011 "Implement capture_screenshot() in alerts.rs"
Task: T012 "Update pipeline.rs alert reset flow"
Task: T013 "Update sns/mod.rs for missing snapshot"

# Remove plotters code (can overlap with T013):
Task: T014 "Remove plot.rs and mod.rs reference"
Task: T015 "Remove plotters from Cargo.toml and font files"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational (T003-T004)
3. Write US1 tests first (T005-T008) — verify they fail
4. Complete Phase 3: User Story 1 implementation (T009-T015)
5. **STOP and VALIDATE**: All US1 tests pass, manual capture test via curl
6. The capture pipeline is fully functional at this point

### Incremental Delivery

1. Setup + Foundational → Infrastructure ready
2. Add User Story 1 (tests + impl) → Test capture pipeline → **MVP complete**
3. Add User Story 2 → systemd + Makefile install → Production-ready
4. Add User Story 3 → Verify non-interference → Hardened
5. Polish → Build verification, test fixes → Release-ready

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- plotters removal (T014-T015) must happen AFTER capture_screenshot is wired in (T011-T012)
- Font files (DejaVuSansCondensed.ttf, NotoSansJP-Bold.otf, NotoSansJP-Bold.ttf) are only used by plot.rs — safe to remove
- Display settings (bandpass filter, spectrogram range, deconvolution) are fetched by the capture page from the data API response (PlotSettings), not passed as URL query parameters
