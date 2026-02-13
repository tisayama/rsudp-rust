# Data Model: WebUI Scroll Sync & Bandpass Filter

**Feature**: 038-webui-scroll-bandpass
**Date**: 2026-02-10

## Entities

### 1. PlotSettings (Backend Web API — extended)

**Location**: `rsudp-rust/src/web/stream.rs`

Existing fields remain unchanged. New fields added:

| Field | Type | Source | Description |
|-------|------|--------|-------------|
| `filter_waveform` | bool | `[plot].filter_waveform` | Whether bandpass filter is enabled for waveform |
| `filter_highpass` | f64 | `[plot].filter_highpass` | Bandpass high-pass cutoff frequency (Hz) |
| `filter_lowpass` | f64 | `[plot].filter_lowpass` | Bandpass low-pass cutoff frequency (Hz) |

Defaults: `filter_waveform: false`, `filter_highpass: 0.7`, `filter_lowpass: 2.0`

### 2. PlotSettings (Frontend — extended)

**Location**: `webui/lib/types.ts`

New fields added to existing `PlotSettings` interface:

| Field | Type | Description |
|-------|------|-------------|
| `filter_waveform` | boolean | Whether bandpass filter is active |
| `filter_highpass` | number | Bandpass high-pass cutoff (Hz) |
| `filter_lowpass` | number | Bandpass low-pass cutoff (Hz) |

### 3. SpectrogramState (Frontend — extended)

**Location**: `webui/src/app/page.tsx` (inline interface)

New field added:

| Field | Type | Description |
|-------|------|-------------|
| `firstColumnTimestamp` | number | Epoch milliseconds of the first stored column. Used to compute any column's absolute time: `firstColumnTimestamp + columnIndex * hopDuration * 1000` |

### 4. Global Time Reference (New concept)

**Location**: `webui/src/app/page.tsx` (computed value)

| Field | Type | Description |
|-------|------|-------------|
| `globalLatestTimestamp` | Date | Maximum of all active channels' `latestTimestamp`. Used as the right edge of the time axis for all channels. |

Computed as: `max(channelTimestampsRef.current[ch])` for all active channels with data.

### 5. ChannelPairCanvas Props (Updated)

**Location**: `webui/components/ChannelPairCanvas.tsx`

Updated prop behavior:

| Prop | Change | Description |
|------|--------|-------------|
| `latestTimestamp` | Semantics change | Now receives `globalLatestTimestamp` instead of per-channel timestamp. All channels share the same value. |
| `spectrogramFirstColumnTimestamp` | New prop | Epoch ms of first stored spectrogram column for timestamp-based rendering. |

## State Transitions

### Backfill → Live Transition

```
State 1: BACKFILL (data < windowSeconds)
  - globalLatestTimestamp advances as packets arrive
  - Waveform right-aligned: samples map to right portion of plot
  - Left portion empty (background only)

State 2: LIVE (data >= windowSeconds)
  - globalLatestTimestamp continues advancing
  - Waveform fills entire plot width
  - Normal left-scrolling behavior
  - Transition is seamless (no state flag needed)
```

The transition happens naturally: as more data accumulates, the time span covered by the data grows until it exceeds `windowSeconds`, at which point the oldest samples are outside the visible window and the plot appears fully filled.

## Relationships

```
rsudp.toml [plot] section
    ├── filter_waveform → Web API PlotSettings.filter_waveform → Frontend PlotSettings.filter_waveform
    ├── filter_highpass → Web API PlotSettings.filter_highpass → Frontend PlotSettings.filter_highpass
    └── filter_lowpass → Web API PlotSettings.filter_lowpass → Frontend PlotSettings.filter_lowpass

page.tsx
    ├── channelTimestampsRef (per-channel) → globalLatestTimestamp (computed max)
    └── globalLatestTimestamp → ChannelPairCanvas.latestTimestamp (all channels same value)

useWebSocket.ts
    └── binary spectrogram timestamp → handleSpectrogramData → SpectrogramState.firstColumnTimestamp

ChannelPairCanvas
    ├── latestTimestamp (global) → rightEdge → leftEdge → waveform x-positions
    ├── latestTimestamp (global) → rightEdge → leftEdge → spectrogram x-positions
    ├── latestTimestamp (global) → rightEdge → leftEdge → alert marker x-positions
    └── latestTimestamp (global) → rightEdge → leftEdge → time axis labels
```
