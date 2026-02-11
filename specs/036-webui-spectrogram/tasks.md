# Tasks: WebUI Spectrogram & rsudp-Compatible Plot

**Input**: Design documents from `/specs/036-webui-spectrogram/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/websocket-protocol.md, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Backend**: `rsudp-rust/src/` (Rust)
- **Frontend**: `webui/` (Next.js / TypeScript)

---

## Phase 1: Setup & Foundational

**Purpose**: Type definitions, utility libraries, and dark theme base required for all subsequent work

- [X] T001 Extend `webui/lib/types.ts` with spectrogram types: add `SpectrogramColumn` (u8 data + timestamp + frequencyBins), `SpectrogramPacket` (parsed binary 0x03 packet), extend `PlotSettings` with `show_spectrogram: boolean`, `spectrogram_freq_min: number`, `spectrogram_freq_max: number`, `spectrogram_log_y: boolean`, add `IntensityIndicatorState` type (visible, maxIntensity, maxClass, triggerTime, resetTime, fadeoutTimer)
- [X] T002 [P] Create `webui/lib/inferno-colormap.ts` with 256-entry RGB lookup table (`INFERNO_RGB: [number, number, number][]`) generated from matplotlib inferno colormap via Python script or pre-embedded constant, plus a helper `infernoToRGBA(u8Value: number): [number, number, number, number]` that returns RGBA with alpha=255
- [X] T003 [P] Create `webui/lib/engineering-format.ts` with SI prefix formatter function `formatEngineering(value: number, unit?: string): string` supporting prefixes μ, m, (none), k, M, G matching rsudp EngFormatter behavior (e.g., 0.00123 → "1.23m", 45000 → "45.0k")
- [X] T004 [P] Apply dark theme base: update `webui/src/app/globals.css` to set `--background: #202530`, `--foreground: rgba(204,204,204,1.0)`, remove light/dark media query toggle (dark-only), update `webui/src/app/layout.tsx` body class for dark background

**Checkpoint**: Shared types, utilities, and base theme ready — user story implementation can begin

---

## Phase 2: User Story 1 — Real-Time Spectrogram Display (Priority: P1) 🎯 MVP

**Goal**: Server computes FFT spectrogram using existing rustfft, sends u8-normalized data via WebSocket, client receives and renders spectrogram in real-time below waveform

**Independent Test**: Connect to streamer simulation, verify spectrogram canvas updates in real-time with inferno colormap showing frequency content; bright band visible during earthquake event

### Backend (Rust)

- [X] T005 [US1] Add `SpectrogramU8` struct (`frequency_bins: usize`, `sample_rate: f64`, `columns: Vec<Vec<u8>>`, `timestamps: Vec<f64>`) and `compute_spectrogram_u8()` function to `rsudp-rust/src/web/plot.rs` that calls existing `compute_spectrogram()`, applies power scaling `(mag_sq / max_mag_sq).powf(0.1) * 255.0` and casts to u8. Add unit test verifying known sine wave input produces expected peak bin at correct frequency index and output range is 0-255
- [X] T006 [US1] Add spectrogram binary packet (type 0x03) serialization function `serialize_spectrogram_packet(channel_id: &str, timestamp_us: i64, sample_rate: f32, spec: &SpectrogramU8) -> Vec<u8>` to `rsudp-rust/src/web/stream.rs` per format in `contracts/websocket-protocol.md`: `[0x03][channelIdLen:u8][channelId:utf8][timestamp:i64le][sampleRate:f32le][frequencyBins:u16le][columnsCount:u16le][data:u8[]]` column-major layout
- [X] T007 [US1] Implement live spectrogram streaming in `rsudp-rust/src/web/stream.rs`: on each `broadcast_waveform()` call, accumulate samples in a per-channel FFT input buffer, when enough samples for a new FFT frame (NFFT=128, step=NFFT-overlap where overlap=90%), call `compute_spectrogram_u8()` on the accumulated window, broadcast the spectrogram packet via the existing broadcast channel after the waveform packet. Add `WsMessage::Spectrogram` variant
- [X] T008 [US1] Implement backfill protocol server-side in `rsudp-rust/src/web/stream.rs`: modify `handle_socket()` to wait for first client text message after connection, parse `BackfillRequest` JSON (`{"type":"BackfillRequest","last_timestamp":"..."}` or no `last_timestamp`), extract data from each `ChannelBuffer` (full buffer or since `last_timestamp`), send waveform binary packets (0x00) + compute and send spectrogram binary packets (0x03) per channel, send `BackfillComplete` JSON with channel list, then begin live streaming from broadcast channel

### Frontend (TypeScript)

- [X] T009 [P] [US1] Update `webui/hooks/useWebSocket.ts`: on WebSocket `onopen`, send `BackfillRequest` JSON (with `last_timestamp` from ref if reconnecting, without if initial). In binary message handler, check first byte: `0x00` = existing waveform handler, `0x03` = new spectrogram handler (parse `channelIdLen`, `channelId`, `timestamp`, `sampleRate`, `frequencyBins`, `columnsCount`, extract u8 data columns). Track `last_timestamp` in useRef, updated on each waveform packet. Add `BackfillComplete` JSON text message handling. Export new spectrogram data via callback prop `onSpectrogramData?: (channelId: string, columns: Uint8Array[], frequencyBins: number, sampleRate: number) => void`
- [X] T010 [P] [US1] Create `webui/lib/SpectrogramRenderer.ts`: class that manages a spectrogram Canvas rendering state. Constructor takes `(canvas: HTMLCanvasElement, frequencyBins: number)`. Method `addColumn(u8Data: Uint8Array)`: map each u8 value through `INFERNO_RGB` LUT to RGBA, write 1-pixel-wide column to ImageData, shift existing canvas content left via `drawImage(canvas, 1, 0, w-1, h, 0, 0, w-1, h)`, `putImageData` new column at right edge. Method `addBulkColumns(columns: Uint8Array[])`: efficiently render multiple columns for backfill. Method `clear()`: fill canvas with black
- [X] T011 [US1] Add temporary spectrogram integration to `webui/src/app/page.tsx`: below each existing WaveformCanvas, add a `<canvas>` element (same width, 1/3 height), instantiate `SpectrogramRenderer` per channel, wire `onSpectrogramData` from useWebSocket to push columns into the corresponding renderer. This validates the end-to-end data flow (backend FFT → WebSocket → client rendering). Will be replaced by ChannelPairCanvas in US2

**Checkpoint**: Spectrogram renders in real-time below waveform; backfill populates spectrogram on page load; inferno colormap visible

---

## Phase 3: User Story 2 — Combined Waveform + Spectrogram Layout (Priority: P1)

**Goal**: Replace separate waveform/spectrogram canvases with a unified ChannelPairCanvas component matching rsudp's paired layout, with channel sorting and shared time axis

**Independent Test**: View WebUI with 1-4 active channels; each shows waveform (2/3 height) above spectrogram (1/3 height) with matching time axis; channels sorted Z→E→N

### Implementation

- [X] T012 [US2] Create `webui/components/ChannelPairCanvas.tsx`: React component with props `{ channelId, buffer: RingBuffer, spectrogramColumns: Uint8Array[], frequencyBins: number, sampleRate: number, windowSeconds: number, autoScale: boolean, alerts: VisualAlertMarker[], settings: PlotSettings }`. Render two stacked canvases: waveform (2/3 height) on top, spectrogram (1/3 height) on bottom. Use `useRef` for both canvas elements and `useEffect` with 30ms `setInterval` render loop
- [X] T013 [US2] Implement waveform rendering in `ChannelPairCanvas.tsx`: dark canvas background (#202530), draw waveform line in #c28285 (pinkish) with lineWidth ~0.45, apply DC offset removal (subtract mean of visible samples), auto-scale Y-axis to min/max with 10% padding (FR-022), draw channel name legend ("EHZ") in upper-left corner with light text, draw Y-axis unit label from deconvolution setting per FR-020 unit table (VEL→"Velocity (m/s)", ACC→"Acceleration (m/s²)", GRAV→"Earth gravity (g)", DISP→"Displacement (m)", CHAN→channel-prefix-based, default→"Counts") with engineering notation via `formatEngineering()`
- [X] T014 [US2] Implement spectrogram rendering in `ChannelPairCanvas.tsx`: instantiate `SpectrogramRenderer` on the spectrogram canvas, forward received spectrogram columns to it, draw Hz frequency Y-axis labels on left side (0 to Nyquist), auto-adjusting color scale per frame (FR-028 — handled server-side in compute_spectrogram_u8 normalization)
- [X] T015 [US2] Update `webui/src/app/page.tsx`: replace WaveformCanvas imports with ChannelPairCanvas, implement channel sorting using `channelSortKey()` from data-model.md (Z=0, E=1, N=2, other=3), remove temporary US1 spectrogram canvases, apply dark theme to page container and header (bg-[#202530], text-gray-300), manage per-channel spectrogram column state as `Record<string, { columns: Uint8Array[], frequencyBins: number }>` updated from `onSpectrogramData` callback
- [X] T016 [US2] Implement shared relative-seconds time axis: all ChannelPairCanvas instances share the same time range (0 to windowSeconds in elapsed seconds). Draw X-axis tick labels (0, 10, 20, ..., 90) only on the bottom-most ChannelPairCanvas with label "Time (seconds)" (FR-026). Suppress X-axis labels on upper channel pairs

**Checkpoint**: rsudp-like layout with paired waveform+spectrogram per channel, channel sorting, dark theme, shared time axis

---

## Phase 4: User Story 3 — Spectrogram Configuration (Priority: P2)

**Goal**: Users can toggle spectrogram on/off, adjust frequency range, and switch to logarithmic Y-axis from the control panel

**Independent Test**: Open control panel, toggle spectrogram off (only waveforms shown), set frequency range to 0.5-10 Hz (spectrogram clips), enable log Y-axis (tick marks at 0.5, 1, 2, 5, 10, 20, 50 Hz)

### Implementation

- [X] T017 [US3] Add spectrogram controls to `webui/components/ControlPanel.tsx`: "Show Spectrogram" toggle switch (default: on), "Freq Min" and "Freq Max" number inputs (default: 0 and 50 Hz), "Log Y-Axis" toggle switch (default: off). Apply dark theme to entire ControlPanel (bg-[#202530] or bg-[#2a2f3d], border-gray-700, text-gray-300, accent colors for toggles). Wire controls to PlotSettings state via `onSettingsChange`
- [X] T018 [US3] Wire spectrogram settings to ChannelPairCanvas.tsx: when `show_spectrogram` is false, hide spectrogram canvas and expand waveform to full height. When `spectrogram_freq_min/max` are set, clip the rendered frequency range in SpectrogramRenderer (only render rows within the frequency range). When `spectrogram_log_y` is true, apply logarithmic mapping to Y-axis pixel positions with tick marks at 0.5, 1, 2, 5, 10, 20, 50 Hz

**Checkpoint**: Spectrogram configuration controls work independently; toggling off hides spectrogram, frequency range clips display, log Y-axis transforms scale

---

## Phase 5: User Story 4 — Event Markers on Both Panels (Priority: P2)

**Goal**: STA/LTA trigger and reset markers appear as vertical dashed lines on both waveform and spectrogram with rsudp-compatible colors

**Independent Test**: Run simulation data that triggers STA/LTA; blue dashed line appears at trigger time, red dashed at reset time on BOTH waveform and spectrogram panels

### Implementation

- [X] T019 [US4] Update event marker rendering in `ChannelPairCanvas.tsx`: change trigger marker color from existing red (#ef4444) to rsudp-compatible blue (#4C8BF5 dashed), change reset marker color from existing green (#10b981) to rsudp-compatible red (#D72638 dashed). Draw markers spanning full height of each panel
- [X] T020 [US4] Extend marker rendering to spectrogram panel in `ChannelPairCanvas.tsx`: currently markers only appear on waveform canvas. Add identical marker drawing logic to spectrogram canvas render loop, using the same timestamp→pixel conversion as waveform. Markers on both panels must align vertically at the same time position

**Checkpoint**: Trigger (blue) and reset (red) markers visible on both waveform and spectrogram panels for all channels

---

## Phase 6: User Story 5 — Event Counter, Header & Intensity Badge (Priority: P3)

**Goal**: Dashboard shows rsudp-style header with station name, event count, branding, and JMA intensity badge during seismic events

**Independent Test**: Observe header shows "Detected Events: 0" initially, counter increments on each trigger. During simulated earthquake, intensity badge appears in top-right with correct JMA color, disappears 30s after reset

### Implementation

- [X] T021 [US5] Add dashboard header to `webui/src/app/page.tsx`: centered title "{STATION} Live Data - Detected Events: {N}" using foreground color on dark background (FR-024). Track event count in state, increment on each AlertStart message. Station name obtained from channel data or config
- [X] T022 [P] [US5] Add "rsudp-rust" branding text in top-left corner of `webui/src/app/page.tsx` header area (FR-025), small font, foreground color (#cccccc) on dark background
- [X] T023 [US5] Create `webui/components/IntensityBadge.tsx`: React component implementing `IntensityIndicatorState` state machine from data-model.md. Hidden by default. On AlertStart → set visible, start tracking max intensity. On Intensity message → update `maxIntensity`/`maxClass` only if higher. On AlertEnd → start 30s fadeout timer. After 30s → hide. If new AlertStart during PostReset → reset and re-track. Badge visual: rounded-corner box with intensity class text (e.g., "3", "5+") in large bold white text, background color from JMA scale (#F2F2FF for 1, #00AAFF for 2, #0041FF for 3, #FAE696 for 4, #FFE600 for 5-, #FF9900 for 5+, #FF2800 for 6-, #A50021 for 6+, #B40068 for 7)
- [X] T024 [US5] Integrate IntensityBadge into `webui/src/app/page.tsx`: position in top-right corner (absolute/fixed), wire AlertStart/AlertEnd/Intensity WebSocket messages (from `lastMessage` or new callbacks) to IntensityBadge state management. Handle edge case: re-trigger during PostReset period (FR-034)

**Checkpoint**: Header shows event count, branding visible, intensity badge appears/updates/disappears correctly during simulated earthquake events

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Remaining dark theme migration, performance validation, buffer sync, end-to-end verification

- [X] T025 [P] Apply dark theme to `webui/components/AlertSettingsPanel.tsx`: replace all light theme classes (bg-white → bg-[#2a2f3d], text-gray-800 → text-gray-300, border-gray-200 → border-gray-700, bg-slate-50 → bg-[#202530], bg-slate-900 → bg-gray-600)
- [X] T026 [P] Apply dark theme to `webui/components/PerformanceMonitor.tsx`: already uses bg-black/80 overlay — verify it works on dark background, adjust border or opacity if needed for visibility
- [X] T027 Sync server-side ChannelBuffer `max_len` with client display window setting in `rsudp-rust/src/web/stream.rs`: ensure `push_segment` max_len parameter equals `window_seconds * sample_rate` from PlotSettings, so buffer retains enough data for the full display window (FR-029). Add `window_seconds` max validation to 300 (FR-035)
- [X] T028 Validate backfill performance: ensure waveform + spectrogram populate within 1 second of WebSocket connection (FR-030) by testing with 90s/4ch default and 300s/4ch max window
- [X] T029 Performance validation: confirm 15+ FPS rendering with 4 channels × (waveform + spectrogram) using PerformanceMonitor (FR-012). Profile and optimize if needed (e.g., reduce render loop frequency, throttle spectrogram column rendering)
- [X] T030 Run quickstart.md verification checklist end-to-end with streamer simulation: dark theme, paired layout, channel sorting, inferno colormap, event markers, title, branding, intensity badge, backfill, spectrogram controls

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (US1)**: Depends on Phase 1 completion (types, LUT, formatter, dark base)
- **Phase 3 (US2)**: Depends on Phase 2 (US1) — needs working spectrogram data pipeline
- **Phase 4 (US3)**: Depends on Phase 3 (US2) — needs ChannelPairCanvas with spectrogram
- **Phase 5 (US4)**: Depends on Phase 3 (US2) — needs ChannelPairCanvas for dual-panel markers
- **Phase 6 (US5)**: Depends on Phase 1 only — header/badge independent of spectrogram
- **Phase 7 (Polish)**: Depends on all user stories being complete

### User Story Dependencies

```
Phase 1 (Setup)
    │
    ├──→ Phase 2 (US1: Spectrogram Data Pipeline) 🎯 MVP
    │        │
    │        ├──→ Phase 3 (US2: Combined Layout)
    │        │        │
    │        │        ├──→ Phase 4 (US3: Configuration)
    │        │        └──→ Phase 5 (US4: Event Markers)
    │        │
    │        └──→ (can validate spectrogram independently)
    │
    └──→ Phase 6 (US5: Header & Badge) ← independent track
                │
                └──→ Phase 7 (Polish) ← after all stories
```

### Within Each User Story

- Backend tasks before frontend integration tasks
- Serialization before streaming
- Data reception before rendering
- Core rendering before configuration/customization

### Parallel Opportunities

**Phase 1**: T002, T003, T004 all run in parallel (different files, no dependencies)

**Phase 2 (US1)**: Backend (T005→T006→T007→T008) and Frontend (T009, T010) tracks in parallel:
```
Backend: T005 → T006 → T007 → T008
                                  ↘
Frontend: T009 → T010 ──────────→ T011 (integration, needs both)
```

**Phase 4+5 (US3+US4)**: Can run in parallel after US2 completes (different concerns, minimal file overlap)

**Phase 6 (US5)**: T021 and T022 can run in parallel. US5 can run in parallel with US3/US4 (independent track)

**Phase 7**: T025, T026 run in parallel (different files)

---

## Parallel Example: User Story 1

```text
# Backend and frontend tracks in parallel:

Track A (Backend):
  T005: Add SpectrogramU8 + compute_spectrogram_u8() to plot.rs
  T006: Add spectrogram packet serialization to stream.rs
  T007: Implement live spectrogram streaming in stream.rs
  T008: Implement backfill protocol in stream.rs

Track B (Frontend — can start immediately, contract-driven):
  T009: Update useWebSocket.ts (BackfillRequest + 0x03 parsing)
  T010: Create SpectrogramRenderer.ts (u8→ImageData rendering)

# Merge point:
  T011: Integration test in page.tsx (needs both tracks complete)
```

---

## Implementation Strategy

### MVP First (Phase 1 + Phase 2 = US1 Only)

1. Complete Phase 1: Setup types, LUT, formatter, dark base
2. Complete Phase 2: US1 backend FFT → WebSocket → frontend renderer
3. **STOP and VALIDATE**: Spectrogram renders in real-time with inferno colormap
4. Backfill works on page refresh (data immediately visible)
5. This proves the entire server-side FFT → u8 → WebSocket → client rendering pipeline

### Incremental Delivery

1. **Phase 1 + 2**: Setup + US1 → Spectrogram data pipeline working (MVP)
2. **Phase 3**: US2 → rsudp-like paired layout with sorting (visual parity)
3. **Phase 4 + 5**: US3 + US4 in parallel → Configuration + event markers (refinement)
4. **Phase 6**: US5 → Header, branding, intensity badge (completeness)
5. **Phase 7**: Polish → remaining dark theme, performance validation, end-to-end test

### Key Architecture Decisions

- **Server-side FFT**: Existing `compute_spectrogram()` in `plot.rs` reused, no new npm FFT dependency
- **u8 normalization**: Power scaling `^(1/10)` + auto-normalize on server → 256-level Inferno LUT on client
- **Spectrogram packet**: Binary type `0x03` with column-major u8 data (~540 bytes/sec/channel)
- **Backfill**: Waveform (0x00) + Spectrogram (0x03) packets sent together per channel

---

## Notes

- [P] tasks = different files, no dependencies on adjacent tasks
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Backend tasks (Rust) and frontend tasks (TypeScript) within a story can often run in parallel
- Total estimated bandwidth: ~844 KB backfill for 90s/4ch, ~2.2 KB/s live for 4ch
