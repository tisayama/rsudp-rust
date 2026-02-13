# Implementation Plan: WebUI Scroll Sync & Bandpass Filter

**Branch**: `038-webui-scroll-bandpass` | **Date**: 2026-02-10 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/038-webui-scroll-bandpass/spec.md`

## Summary

Replace index-based rendering with unified timestamp-based rendering for both waveform and spectrogram. This single architectural change solves three problems: right-aligned backfill (US1), synchronized channel scrolling (US2), and waveform/spectrogram alignment (US3). Additionally, expose bandpass filter settings from the backend and render "Bandpass" and "Range" labels on the canvas (US4).

**Core insight**: Currently, waveform uses sample-index positioning, spectrogram uses column-index positioning, and alert markers use wall-clock positioning — three independent time references that inevitably drift. By using a single shared timestamp (the global maximum across all channels) as the right edge of all plots, and computing every element's x-position from its absolute timestamp, all three rendering systems become inherently synchronized.

## Technical Context

**Language/Version**: Rust 1.7x (Backend), TypeScript / Next.js 14+ (Frontend)
**Primary Dependencies**: `axum` (REST API), `tokio` (async), `serde` (serialization); React, Canvas 2D API (frontend)
**Storage**: In-memory (RingBuffer for waveform, array for spectrogram columns)
**Testing**: `npm run build` (TypeScript compilation), Docker Compose visual testing
**Target Platform**: Web browser (Chrome/Firefox), Linux server (backend)
**Project Type**: Web application (Rust backend + Next.js frontend)
**Performance Goals**: 30 FPS canvas rendering, smooth scrolling
**Constraints**: No changes to WebSocket binary protocol format; no new dependencies
**Scale/Scope**: 4-channel seismograph display, single user

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. 安定性と信頼性 | PASS | Timestamp-based rendering is more correct than index-based; eliminates drift |
| II. 厳密なテスト | PASS | Visual testing via Docker Compose with quickstart scenarios |
| III. 高いパフォーマンス | PASS | Rendering approach unchanged (30 FPS); no performance regression expected |
| IV. コードの明瞭性と保守性 | PASS | Unified time reference simplifies rendering logic; eliminates three separate positioning systems |
| V. 日本語による仕様策定 | PASS | Spec written per workflow |
| VI. 標準技術スタック | PASS | Next.js + WebSocket + Canvas 2D (existing stack) |
| VII. 自己検証の義務 | PASS | Docker Compose visual verification before commit |
| VIII. ブランチ運用 | PASS | Feature branch 038-webui-scroll-bandpass |

## Project Structure

### Documentation (this feature)

```text
specs/038-webui-scroll-bandpass/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── api-settings.md
└── tasks.md
```

### Source Code (files to modify)

```text
rsudp-rust/
├── src/
│   ├── main.rs                    # Copy filter settings to WebState
│   └── web/
│       └── stream.rs              # Add filter fields to Web API PlotSettings

webui/
├── hooks/
│   └── useWebSocket.ts            # Extract spectrogram batch timestamp
├── lib/
│   └── types.ts                   # Add filter fields to PlotSettings interface
├── components/
│   └── ChannelPairCanvas.tsx      # Timestamp-based rendering, bandpass/range labels
└── src/
    └── app/
        └── page.tsx               # Global timestamp, spectrogram timestamp tracking
```

**Structure Decision**: Existing web application structure. Changes span both backend (2 files) and frontend (5 files). No new files created — all changes are modifications to existing files.

## Implementation Strategy

### Phase 1: Foundational — Global Timestamp & Spectrogram Timestamps

**Goal**: Establish the shared time reference infrastructure.

1. **page.tsx**: Compute `globalLatestTimestamp` from per-channel timestamps. Pass to all ChannelPairCanvas as `latestTimestamp`.
2. **useWebSocket.ts**: Extract the spectrogram batch timestamp from the binary protocol (currently skipped at `offset += 8`). Pass it to `handleSpectrogramData` callback.
3. **page.tsx**: Add `firstColumnTimestamp` to `SpectrogramState`. Update it on batch receipt and column trimming.
4. **ChannelPairCanvas.tsx**: Add `spectrogramFirstColumnTimestamp` prop.

### Phase 2: US1+US2+US3 — Unified Timestamp-Based Rendering

**Goal**: Replace all three independent positioning systems with a single timestamp-based approach.

5. **ChannelPairCanvas.tsx — Waveform**: Replace `x = LEFT_MARGIN + (i / (windowSeconds * sampleRate)) * plotWidth` with timestamp-based computation. Use `latestTimestamp` (now global) as right edge, compute each sample's time backwards from channel's latest timestamp.
6. **ChannelPairCanvas.tsx — Spectrogram**: Replace column-index-per-pixel loop with column-iteration loop using `firstColumnTimestamp + colIdx * hopDuration * 1000` for x-positioning.
7. **ChannelPairCanvas.tsx — Alert Markers**: Replace `new Date()` with `latestTimestamp` (global) as the time reference. Use same `(alertTime - leftEdge) / windowMs * plotWidth` formula.

### Phase 3: US4 — Bandpass Filter & Range Labels

**Goal**: Display filter metadata on the canvas.

8. **stream.rs**: Add `filter_waveform`, `filter_highpass`, `filter_lowpass` to Web API PlotSettings.
9. **main.rs**: Copy filter settings from config to WebState during initialization.
10. **types.ts**: Add filter fields to frontend PlotSettings interface.
11. **ChannelPairCanvas.tsx**: Render "Bandpass (X - Y Hz)" label in waveform lower-left (only when `filter_waveform` is true). Render "Range (X - Y Hz)" label in spectrogram lower-left.

### Phase 4: Polish

12. Visual verification against all quickstart scenarios.

## Key Design Decisions

### D1: Why a single global timestamp instead of per-channel?

Per-channel timestamps cause channels to scroll at different rates because packets arrive at slightly different times. A global max ensures all channels share the same time window. Individual channels that have less data will simply show empty space on the left (naturally right-aligned).

### D2: Why timestamp-based rendering instead of index-based?

Index-based rendering (`i / totalSamples`) assumes the buffer is always full. During backfill, it compresses data into the left portion. Timestamp-based rendering places each sample at its correct absolute time position, which naturally right-aligns partial data and ensures waveform/spectrogram/alerts all share the same coordinate system.

### D3: Why `firstColumnTimestamp` instead of per-column timestamps?

Spectrogram columns are evenly spaced by `hopDuration`. Storing a single base timestamp and computing each column's time as `base + index * hop` is O(1) storage vs O(n) for per-column timestamps. When trimming, we simply add `trimCount * hopDuration * 1000` to the base.

### D4: Waveform sample-to-timestamp computation

Each waveform sample's timestamp is computed as:
```
sampleTime = channelLatestTimestamp - ((samples.length - 1 - i) / sampleRate) * 1000
```
Where `channelLatestTimestamp` is the per-channel timestamp (not global), and `i` is the sample index in the getTail result. The newest sample (last in array) corresponds to `channelLatestTimestamp`, working backwards at `1/sampleRate` intervals.

The x-position is then:
```
rightEdge = globalLatestTimestamp
leftEdge = rightEdge - windowSeconds * 1000
x = LEFT_MARGIN + ((sampleTime - leftEdge) / (windowSeconds * 1000)) * plotWidth
```

If `channelLatestTimestamp < globalLatestTimestamp` (this channel is behind), the waveform appears slightly to the left of the right edge — which is the correct behavior.

## Complexity Tracking

No constitution violations. No complexity justifications needed.
