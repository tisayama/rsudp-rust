# Implementation Plan: rsudp Plot Timestamp & Grid Fix

**Branch**: `015-fix-plot-timestamp` | **Date**: 2025-12-22 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/015-fix-plot-timestamp/spec.md`

## Summary
Refine the alert plot generation logic to perfectly match `rsudp` standards. This involves replacing relative time offsets with absolute UTC timestamps on the X-axis and removing the grid lines from the waveform plots.

## Technical Context

**Language/Version**: Rust 1.7x
**Primary Dependencies**: `plotters`, `chrono`
**Storage**: N/A
**Testing**: `test_plot_comparison.rs` (visual verification)
**Target Platform**: Linux / Cross-platform
**Project Type**: Library Enhancement
**Performance Goals**: N/A (formatting overhead is negligible)
**Constraints**: Must maintain sub-second alignment between waveform and spectrogram.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. 安定性**: ✅ Only visual logic changes; no impact on data ingestion.
- **II. 厳密なテスト**: ✅ Visual regression via existing comparison tests.
- **III. 高いパフォーマンス**: ✅ Uses efficient `plotters` formatting.
- **IV. 明瞭性**: ✅ Aligns with `rsudp` standards.
- **V. 日本語仕様**: ✅ Specification is in Japanese.

## Project Structure

### Documentation (this feature)

```text
specs/015-fix-plot-timestamp/
├── plan.md              # This file
├── research.md          # X-axis formatting research
├── data-model.md        # Logic flow
└── quickstart.md        # Verification steps
```

### Source Code

```text
rsudp-rust/
├── src/
│   ├── web/
│   │   └── plot.rs      # Modified: X-axis formatting and grid removal
```

**Structure Decision**: Continue using `src/web/plot.rs` as the primary engine for image generation.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | | |