# Tasks: WebUI Scroll Sync & Bandpass Filter

**Input**: Design documents from `/specs/038-webui-scroll-bandpass/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Organization**: Tasks grouped by phase. US1/US2/US3 are implemented together (same code change: unified timestamp-based rendering) but labeled individually. US4 is independent.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Foundational — Timestamp Infrastructure

**Purpose**: Establish the shared time reference and spectrogram timestamp tracking that all three P1 user stories depend on.

**CRITICAL**: No rendering changes until this phase is complete.

- [X] T001 Extract spectrogram batch timestamp from binary protocol in webui/hooks/useWebSocket.ts: at the `offset += 8` line (currently skipping timestamp), read the i64 little-endian value as microseconds, convert to milliseconds (`timestamp_us / 1000`), and pass it to the `onSpectrogramData` callback as a new first parameter `batchTimestamp: number`. Update the callback type signature accordingly.
- [X] T002 Add `firstColumnTimestamp` field to the `SpectrogramState` interface in webui/src/app/page.tsx. Update `handleSpectrogramData` callback to accept `batchTimestamp: number` as first parameter. On first batch for a channel (no existing columns), set `firstColumnTimestamp = batchTimestamp`. When trimming N columns from the left, update `firstColumnTimestamp += N * hopDuration * 1000`. Ensure `firstColumnTimestamp` is preserved across appends (sequential columns).
- [X] T003 Compute `globalLatestTimestamp` in webui/src/app/page.tsx: after updating `channelTimestampsRef`, compute the maximum Date across all active channels' timestamps. Store as a separate state/ref value. Pass this `globalLatestTimestamp` as the `latestTimestamp` prop to all ChannelPairCanvas components. Also pass the per-channel timestamp as a new `channelLatestTimestamp` prop so each channel can compute its sample times.
- [X] T004 Update ChannelPairCanvasProps interface in webui/components/ChannelPairCanvas.tsx: add `channelLatestTimestamp: Date | null` (per-channel timestamp for sample time computation) and `spectrogramFirstColumnTimestamp: number` (epoch ms of first stored spectrogram column). Accept both in destructuring. Add corresponding refs (`channelLatestTimestampRef`, `spectrogramFirstColumnTimestampRef`) for use in the 30 FPS render loop.
- [X] T005 Pass new props from webui/src/app/page.tsx to each ChannelPairCanvas: `channelLatestTimestamp={channelTimestamps[id] || null}` and `spectrogramFirstColumnTimestamp={spectrogramData[id]?.firstColumnTimestamp || 0}`.

**Checkpoint**: Timestamp infrastructure complete — global timestamp flows to all channels, spectrogram batch timestamps extracted and tracked.

---

## Phase 2: US1 + US2 + US3 — Unified Timestamp-Based Rendering (Priority: P1)

**Goal**: Replace three independent positioning systems (sample-index, column-index, wall-clock) with a single timestamp-based system. This simultaneously solves right-aligned backfill (US1), synchronized scrolling (US2), and waveform/spectrogram alignment (US3).

**Independent Test**: Start WebUI fresh → data right-aligned during backfill (US1). Multiple channels scroll in sync (US2). Alert markers align between waveform and spectrogram (US3).

### Implementation

- [X] T006 [US1][US2] Replace waveform x-position formula in webui/components/ChannelPairCanvas.tsx: compute `rightEdge = latestTimestamp.getTime()` (global), `leftEdge = rightEdge - windowSeconds * 1000`. For each sample `i`, compute `sampleTime = channelLatestTimestampRef.current.getTime() - ((samples.length - 1 - i) / sampleRate) * 1000`. Map to pixel: `x = LEFT_MARGIN + ((sampleTime - leftEdge) / (windowSeconds * 1000)) * plotWidth`. Skip samples outside `[leftEdge, rightEdge]`. This naturally right-aligns data during backfill (empty space on left) and synchronizes all channels (same rightEdge).
- [X] T007 [US3] Replace spectrogram rendering in webui/components/ChannelPairCanvas.tsx: change from pixel-driven loop (`for x in 0..plotWidth → colIdx`) to column-driven loop. For each stored column `ci`, compute `colTime = spectrogramFirstColumnTimestampRef.current + (startCol + ci) * hopDurationRef.current * 1000`. If `colTime < leftEdge || colTime > rightEdge`, skip. Compute `x = Math.round(LEFT_MARGIN + ((colTime - leftEdge) / (windowSeconds * 1000)) * plotWidth)` and `nextX` for column width. Draw the column's frequency bins as vertical pixel strips from `x` to `nextX`. Use same `leftEdge`/`rightEdge` as waveform for alignment.
- [X] T008 [US3] Replace alert marker positioning in the `drawAlertMarkers` function in webui/components/ChannelPairCanvas.tsx: change the function signature to accept `rightEdge: number` (ms) instead of computing `const now = new Date()`. Replace `const diffMs = now.getTime() - alertTime.getTime()` / `const diffSec = diffMs / 1000` / `x = xOffset + plotWidth - (diffSec / windowSeconds) * plotWidth` with `x = xOffset + ((alertTime.getTime() - leftEdge) / (windowSeconds * 1000)) * plotWidth` where `leftEdge = rightEdge - windowSeconds * 1000`. Update both call sites (waveform and spectrogram) to pass `latestTimestamp.getTime()` as `rightEdge`.

**Checkpoint**: All three P1 stories verified — right-aligned backfill, synchronized channels, aligned waveform/spectrogram.

---

## Phase 3: US4 — Bandpass Filter & Range Labels (Priority: P2)

**Goal**: Display "Bandpass (X - Y Hz)" on waveform and "Range (X - Y Hz)" on spectrogram.

**Independent Test**: Open WebUI → see "Bandpass" label on waveform (when filter enabled) and "Range" label on spectrogram, matching configured values.

### Implementation

- [X] T009 [P] [US4] Add `filter_waveform: bool`, `filter_highpass: f64`, `filter_lowpass: f64` fields to the Web API `PlotSettings` struct in rsudp-rust/src/web/stream.rs. Add defaults in the Default impl: `filter_waveform: false`, `filter_highpass: 0.7`, `filter_lowpass: 2.0`. The `/api/settings` endpoint already serializes all struct fields via Serde — no route changes needed.
- [X] T010 [P] [US4] Copy filter settings from config to WebState in rsudp-rust/src/main.rs: in the initialization block where `web_state.settings` is configured, add `plot_settings.filter_waveform = settings.plot.filter_waveform;`, `plot_settings.filter_highpass = settings.plot.filter_highpass;`, `plot_settings.filter_lowpass = settings.plot.filter_lowpass;`.
- [X] T011 [P] [US4] Add `filter_waveform: boolean`, `filter_highpass: number`, `filter_lowpass: number` fields to the frontend `PlotSettings` interface in webui/lib/types.ts. Also add defaults in `DEFAULT_SETTINGS` in webui/src/app/page.tsx: `filter_waveform: false`, `filter_highpass: 0.7`, `filter_lowpass: 2.0`.
- [X] T012 [US4] Render "Bandpass" and "Range" labels in webui/components/ChannelPairCanvas.tsx: In the waveform rendering section (after the border), if `settings.filter_waveform` is true, draw `Bandpass (${settings.filter_highpass} - ${settings.filter_lowpass} Hz)` in the lower-left corner of the waveform data area (at approximately `x = LEFT_MARGIN + 5, y = waveformHeight - 5`) with a small semi-transparent background for readability. In the spectrogram rendering section (after the border), draw `Range (${settings.spectrogram_freq_min} - ${settings.spectrogram_freq_max} Hz)` in the lower-left corner of the spectrogram data area. Use `font = '9px Arial'`, `fillStyle = FG_COLOR`.

**Checkpoint**: Bandpass and Range labels visible and correct. Bandpass hidden when `filter_waveform` is false.

---

## Phase 4: Polish & Verification

**Purpose**: Build verification and visual testing

- [X] T013 Build verification: run `npm run build` in webui/ to verify TypeScript compilation passes with no errors. Run `cargo build` in rsudp-rust/ to verify Rust compilation.
- [ ] T014 Visual verification: run `docker compose up` and verify all 7 quickstart.md scenarios pass — right-aligned backfill (S1), synchronized scrolling (S2), waveform/spectrogram alignment (S3), bandpass label (S4), range label (S5), bandpass hidden when disabled (S6), smooth backfill-to-live transition (S7).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 1)**: No dependencies — can start immediately
- **US1+US2+US3 (Phase 2)**: Depends on Phase 1 (needs global timestamp infrastructure)
- **US4 (Phase 3)**: Independent — backend tasks (T009, T010) can run in parallel with Phase 1/2. Frontend tasks (T011, T012) can run after Phase 2 since T012 modifies ChannelPairCanvas.tsx.
- **Polish (Phase 4)**: Depends on all prior phases being complete

### User Story Dependencies

- **US1 (Right-Aligned Backfill)**: Depends on Phase 1 (global timestamp). Implemented in T006.
- **US2 (Sync Scrolling)**: Depends on Phase 1 (global timestamp). Implemented in T006 (same code change as US1).
- **US3 (Waveform/Spectrogram Alignment)**: Depends on Phase 1 (spectrogram timestamps). Implemented in T007, T008.
- **US4 (Bandpass Labels)**: Backend tasks (T009, T010) fully independent. Frontend tasks depend on ChannelPairCanvas.tsx stability.

### Within Each Phase

- Phase 1: T001 → T002 (T002 depends on T001's callback change), T003 → T004 → T005 (sequential prop propagation)
- Phase 2: T006 → T007 → T008 (sequential, same file)
- Phase 3: T009, T010, T011 can all run in parallel [P] (different files). T012 depends on T011.

### Parallel Opportunities

- T001 and T003 can start in parallel (different files: useWebSocket.ts vs page.tsx)
- T009, T010, T011 can all run in parallel (stream.rs, main.rs, types.ts — different files)
- T009+T010 (backend) can run in parallel with Phase 1+2 (frontend)

---

## Implementation Strategy

### MVP First (US1 + US2 + US3)

1. Complete Phase 1: Foundational (timestamp infrastructure)
2. Complete Phase 2: US1+US2+US3 (timestamp-based rendering)
3. **STOP and VALIDATE**: Right-aligned backfill, synchronized scrolling, aligned plots
4. Deploy/demo if ready — this delivers the three most impactful visual fixes

### Incremental Delivery

1. Phase 1 → Timestamp infrastructure ready
2. Phase 2 → Three P1 stories complete → Most impactful visual improvements done
3. Phase 3 → Bandpass/Range labels → Display parity with rsudp reference
4. Phase 4 → Build + visual verification → Feature complete

---

## Notes

- US1, US2, US3 are fundamentally interconnected: all three are solved by the same architectural change (timestamp-based rendering with a shared global time reference). They cannot be meaningfully separated into independent phases.
- All Phase 2 changes are in webui/components/ChannelPairCanvas.tsx — execute sequentially to avoid conflicts.
- Phase 3 backend tasks (T009, T010) modify Rust files and can run at any time without affecting frontend work.
- The spectrogram rendering change (T007) is the most complex task — changing from a pixel-driven ImageData loop to a column-driven approach requires careful handling of the column-to-pixel width mapping.
- Alert marker alignment (T008) is a relatively small change but critical for visual verification of US3.
