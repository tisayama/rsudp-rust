# Implementation Plan: rsudp Plot Compatibility

**Branch**: `011-rsudp-plot-compatibility` | **Date**: 2025-12-19 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/011-rsudp-plot-compatibility/spec.md`

## Summary
Upgrade the alert image generation to produce professional, `rsudp`-compatible plots. This includes adding a spectrogram view below the waveform, using a vertical stack layout, and matching `rsudp`'s color schemes and styling using `plotters`.

## Technical Context

**Language/Version**: Rust 1.7x
**Primary Dependencies**: `plotters` (Plotting), `rustfft` (Spectrogram), `colorous` (Colormaps, if needed)
**Storage**: Local filesystem (existing `alerts/` directory)
**Testing**: Unit tests for spectrogram PSD calculation
**Target Platform**: Linux (Server)
**Project Type**: Library Enhancement (rsudp-rust)
**Performance Goals**: Image generation < 3s (background task)
**Constraints**: Must match `rsudp` default look (NFFT=256, noverlap=128).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Stability**: ✅ Spectrogram calculation is computationally intensive but will run in a separate `tokio::spawn` task to avoid blocking the ingestion pipeline.
- **II. Rigorous Testing**: ✅ Will add unit tests to verify FFT output against known signals (e.g., sine wave).
- **III. High Performance**: ✅ `plotters` and `rustfft` are highly optimized.
- **IV. Clarity**: ✅ Plotting logic will be encapsulated in a dedicated module (`src/web/plot.rs`) to keep `alerts.rs` clean.
- **V. Japanese Spec**: ✅ Specification exists in Japanese.

## Project Structure

### Documentation (this feature)

```text
specs/011-rsudp-plot-compatibility/
├── plan.md              # This file
├── research.md          # FFT and Styling research
├── data-model.md        # Plot config entities
└── tasks.md             # Execution steps
```

### Source Code

```text
rsudp-rust/
├── src/
│   ├── web/
│   │   ├── plot.rs      # NEW: Dedicated plotting module (Waveform + Spectrogram)
│   │   ├── alerts.rs    # Modified: Calls plot.rs instead of inline plotting
│   │   └── mod.rs
│   └── intensity/
│       └── filter.rs    # Reuse existing FFT logic if possible? (Likely need separate for Spectrogram)
```

**Structure Decision**: Extract plotting logic from `alerts.rs` into `plot.rs` because it's becoming complex (waveform + spectrogram + multi-channel layout).

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Manual FFT Windowing | To match `specgram` defaults | `rustfft` is low-level; no high-level crate matches Matplotlib exactly. |
| Custom Colormap Logic | To match `viridis` | Default `plotters` colormaps might not match `rsudp` exactly. |