# Implementation Plan: Intensity Display on Alert Plots

**Branch**: `012-intensity-on-plot` | **Date**: 2025-12-21 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/012-intensity-on-plot/spec.md`

## Summary
Enhance the `rsudp`-compatible alert plots by adding a high-visibility Japanese seismic intensity (Shindo) badge in the top-right corner. The badge will use `Noto Sans Japanese` for professional typography and the official JMA color palette for intuitive scale communication.

## Technical Context

**Language/Version**: Rust 1.7x
**Primary Dependencies**: `plotters` (with `ab_glyph`), `rustfft`, `chrono`
**Storage**: Embedded `NotoSansJP-Bold.ttf` (via `include_bytes!`)
**Testing**: Visual verification via `test_plot_comparison.rs`
**Target Platform**: Linux / Cross-platform
**Project Type**: Library Enhancement
**Performance Goals**: <100ms overhead for intensity calculation and box rendering
**Constraints**: Must work without system font dependencies.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. 安定性**: ✅ Reuses existing robust `JmaFilter` logic for calculation.
- **II. 厳密なテスト**: ✅ Includes visual regression testing via generated PNGs.
- **III. 高いパフォーマンス**: ✅ Calculation is performed only once per snapshot; font is pre-baked for fast access.
- **IV. 明瞭性**: ✅ Modularized theme and label logic in `plot.rs`.
- **V. 日本語仕様**: ✅ Specification and labels are in Japanese.
- **VI. 標準技術スタック**: ✅ Follows existing Rust backend architecture.

## Project Structure

### Documentation (this feature)

```text
specs/012-intensity-on-plot/
├── plan.md              # This file
├── research.md          # Font and color research
├── data-model.md        # Shindo mapping and rendering entities
└── tasks.md             # Implementation steps
```

### Source Code

```text
rsudp-rust/
├── src/
│   ├── web/
│   │   ├── plot.rs      # Modified: Add draw_intensity_box and font registration
│   │   └── mod.rs
│   └── resources/
│       └── NotoSansJP-Bold.ttf # NEW: Font asset
```

**Structure Decision**: Place the `.ttf` asset in a new `resources/` folder to separate binary assets from source code.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Binary Embedding | Zero-dependency portability | System font lookup is unreliable on minimal Linux distros. |
| Windowed JMA Calculation | Accuracy | Reusing real-time calculated max might differ from what's visually present in the 90s crop. |