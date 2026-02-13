# Research: WebUI Scroll Sync & Bandpass Filter

**Feature**: 038-webui-scroll-bandpass
**Date**: 2026-02-10

## R1: Waveform X-Position Mapping (Root Cause of Left-Alignment)

**Decision**: Replace index-based x-positioning with timestamp-based x-positioning for waveform rendering.

**Rationale**: The current formula `x = LEFT_MARGIN + (i / (windowSeconds * sampleRate)) * plotWidth` always maps sample index 0 to the left edge. During backfill when `buffer.length < windowSeconds * sampleRate`, the waveform is compressed into the left portion of the plot. A timestamp-based approach naturally right-aligns data because each sample's time position is computed relative to the right edge (latest timestamp).

**Current behavior**:
- `RingBuffer.getTail(count)` returns `min(count, buffer.length)` samples
- Sample `i=0` always maps to `LEFT_MARGIN` (pixel 70)
- During backfill: waveform occupies `samples.length / (windowSeconds * sampleRate)` fraction of plot width, growing from left

**New formula**:
```
rightEdge = globalLatestTimestamp (ms)
leftEdge = rightEdge - windowSeconds * 1000
sampleTime = channelLatestTimestamp - ((samples.length - 1 - i) / sampleRate) * 1000
x = LEFT_MARGIN + ((sampleTime - leftEdge) / (windowSeconds * 1000)) * plotWidth
```

**Alternatives considered**:
- Padding `getTail()` result with zeros on the left → Rejected: requires modifying RingBuffer, doesn't solve timestamp alignment
- Offsetting the waveform start x-position → Rejected: doesn't solve synchronization with spectrogram

---

## R2: Channel Scroll Synchronization

**Decision**: Compute a single `globalLatestTimestamp` as the maximum across all active channels' latest timestamps. Pass this to all ChannelPairCanvas components.

**Rationale**: Currently each channel has its own `latestTimestamp` updated when its waveform packets arrive. Since packets arrive at slightly different times per channel, each channel's time axis has a different right edge. Using a global maximum ensures all channels share the same time window.

**Current behavior** (page.tsx):
- `channelTimestampsRef.current[channel_id] = latestTs` — per-channel tracking
- Each ChannelPairCanvas receives its own `channelTimestamps[id]` as `latestTimestamp`

**New behavior**:
- Continue tracking per-channel timestamps (needed for data extent detection)
- Compute `globalLatestTimestamp = max(channelTimestampsRef.current[ch] for ch in active_channels)`
- Pass `latestTimestamp={globalLatestTimestamp}` to all ChannelPairCanvas components
- All channels share the same `[leftEdge, rightEdge]` time window

**Alternatives considered**:
- Using `Date.now()` as the global reference → Rejected: creates drift between data timestamps and wall clock; data would appear to lag
- Using the first channel's timestamp → Rejected: arbitrary choice; may not be the most recent

---

## R3: Waveform/Spectrogram Alignment (Root Cause Analysis)

**Decision**: Use timestamp-based x-positioning for spectrogram columns (matching the waveform approach from R1). Extract the batch timestamp from the binary protocol and track per-column timestamps.

**Root cause of drift**:
1. Waveform uses sample-index-based positioning → independent of actual timestamps
2. Spectrogram uses column-index-based positioning: `colIdx = Math.floor((x / plotWidth) * maxVisibleCols)` → assumes uniform column spacing
3. Alert markers use `new Date()` (wall clock) → yet another time reference
4. Three separate time reference systems → inevitable drift

**Key finding — Binary protocol sends timestamps but frontend discards them**:
- Binary format includes `timestamp_us: i64` per spectrogram batch
- Frontend skips it: `offset += 8; // timestamp i64le (skip for rendering)`
- `SpectrogramU8.timestamps` exists in Rust but is NOT serialized per-column

**New approach — Track `firstColumnTimestamp` in SpectrogramState**:
```typescript
interface SpectrogramState {
  columns: Uint8Array[];
  frequencyBins: number;
  sampleRate: number;
  hopDuration: number;
  totalReceived: number;
  firstColumnTimestamp: number; // ms epoch of the first stored column
}
```

- Extract batch timestamp from binary protocol in `useWebSocket.ts`
- On first batch: `firstColumnTimestamp = batchTimestamp`
- On append: no change to firstColumnTimestamp (columns are sequential)
- On trim (removing N columns from left): `firstColumnTimestamp += N * hopDuration * 1000`
- Column `i`'s time = `firstColumnTimestamp + i * hopDuration * 1000`

**Rendering formula** (same as waveform R1):
```
rightEdge = globalLatestTimestamp
leftEdge = rightEdge - windowSeconds * 1000
for each stored column i:
  colTime = firstColumnTimestamp + i * hopDuration * 1000
  if colTime < leftEdge or colTime > rightEdge: skip
  x = LEFT_MARGIN + ((colTime - leftEdge) / (windowSeconds * 1000)) * plotWidth
```

**Alert markers also aligned**: Replace `new Date()` with `globalLatestTimestamp` for consistent positioning.

**Alternatives considered**:
- Store per-column timestamps array → Rejected: memory overhead, firstColumnTimestamp + hopDuration arithmetic is sufficient
- Use waveform's latestTimestamp as proxy for spectrogram → Rejected: not accurate enough; spectrogram and waveform may have different latencies

---

## R4: Backend Bandpass Filter Settings Gap

**Decision**: Add `filter_waveform`, `filter_highpass`, and `filter_lowpass` fields to the Web API PlotSettings struct in `stream.rs`. Copy values from config PlotSettings during initialization.

**Root cause**: Two separate `PlotSettings` structs exist:
1. **Config PlotSettings** (`settings.rs` lines 75-116): Has `filter_waveform: bool`, `filter_highpass: f64`, `filter_lowpass: f64`, `filter_corners: u32`
2. **Web API PlotSettings** (`web/stream.rs` lines 21-30): Has `scale`, `window_seconds`, `save_pct`, etc. — NO filter fields

The filter fields are loaded from `rsudp.toml` into the config struct but never transferred to `WebState.settings`.

**Implementation**:
1. Add to Web API PlotSettings in `stream.rs`:
   ```rust
   pub filter_waveform: bool,
   pub filter_highpass: f64,
   pub filter_lowpass: f64,
   ```
2. In `main.rs` initialization block, copy from config:
   ```rust
   plot_settings.filter_waveform = settings.plot.filter_waveform;
   plot_settings.filter_highpass = settings.plot.filter_highpass;
   plot_settings.filter_lowpass = settings.plot.filter_lowpass;
   ```
3. Add to frontend `PlotSettings` type in `types.ts`
4. The `/api/settings` endpoint already serializes all fields — no route changes needed

**Alternatives considered**:
- Create separate `/api/filter-config` endpoint → Rejected: filter settings conceptually belong with plot settings; adding a separate endpoint is unnecessary complexity
- Pass filter values via WebSocket messages → Rejected: settings are static configuration, not streaming data

---

## R5: Spectrogram Column Rendering Approach

**Decision**: Change spectrogram rendering from column-index-per-pixel loop to column-iteration loop with timestamp-based pixel mapping.

**Current approach** (pixel-driven):
```typescript
for (let x = 0; x < plotWidth; x++) {
  const colIdx = Math.floor((x / plotWidth) * maxVisibleCols);
  // render pixel column from colIdx
}
```
Problem: Maps every pixel to a column, wastes work when columns are sparse (backfill), misaligns when column count doesn't match expectation.

**New approach** (column-driven):
```typescript
for (let ci = 0; ci < visibleCols.length; ci++) {
  const colTime = firstColumnTimestamp + (startCol + ci) * hopDuration * 1000;
  if (colTime < leftEdge || colTime > rightEdge) continue;
  const x = Math.round(LEFT_MARGIN + ((colTime - leftEdge) / (windowSeconds * 1000)) * plotWidth);
  const nextX = Math.round(LEFT_MARGIN + ((colTime + hopDuration * 1000 - leftEdge) / (windowSeconds * 1000)) * plotWidth);
  // render column data from x to nextX (fill width)
}
```

This is naturally right-aligned (same as waveform R1) and ensures pixel-perfect alignment between waveform and spectrogram.

**Alternatives considered**:
- Keep pixel-driven loop but adjust `maxVisibleCols` during backfill → Rejected: doesn't solve timestamp misalignment with waveform
- Use ImageData for the entire plotWidth then translate → Rejected: wasteful for partial data
