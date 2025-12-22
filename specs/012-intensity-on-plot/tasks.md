# Tasks: Intensity Display on Alert Plots

**Feature Branch**: `012-intensity-on-plot`
**Implementation Strategy**: Integrate windowed JMA intensity calculation into the plotting engine and overlay a high-visibility badge using embedded Japanese fonts.

## Phase 1: Setup

- [X] T001 Create `rsudp-rust/src/resources/` directory
- [X] T002 Download/Place `NotoSansJP-Bold.ttf` in `rsudp-rust/src/resources/` (from https://xfonts.pro/font/Noto-Sans-JP-Bold.html)
- [X] T003 [P] Verify `plotters` in `rsudp-rust/Cargo.toml` has `ab_glyph` and `bitmap_backend` features enabled

## Phase 2: Foundational (Calculation & Theming)

- [X] T004 [P] Define `JMA_COLOR_PALETTE` mapping Shindo levels to RGB colors in `rsudp-rust/src/web/plot.rs`
- [X] T005 Implement `calculate_window_max_intensity` helper in `rsudp-rust/src/web/plot.rs` using `JmaFilter` from `intensity` module

## Phase 3: [US1] High-Visibility Intensity Display (Priority: P1)

**Goal**: Draw a large colored box with white Shindo label in the top-right corner.
**Independent Test**: Generate a plot using `test_plot_comparison.rs` and verify the box color matches the calculated intensity.

- [X] T006 [US1] Implement `draw_intensity_badge` function in `rsudp-rust/src/web/plot.rs` (styled rectangle + text overlay)
- [X] T007 [US1] Integrate `draw_intensity_badge` into `draw_rsudp_plot` in `rsudp-rust/src/web/plot.rs`

## Phase 4: [US2] Professional Japanese Typography (Priority: P2)

**Goal**: Use Noto Sans Japanese for the "震度" label to ensure correct and beautiful rendering.
**Independent Test**: Verify the "震度" characters are rendered correctly in the generated PNG without blocky fallbacks.

- [X] T008 [US2] Embed `NotoSansJP-Bold.ttf` into the binary using `include_bytes!` in `rsudp-rust/src/web/plot.rs`
- [X] T009 [US2] Register and apply the embedded Noto Sans font for the intensity label in `rsudp-rust/src/web/plot.rs`

## Phase 5: Polish & Validation

- [X] T010 [P] Update `rsudp-rust/tests/test_plot_comparison.rs` to include intensity display in the automated visual check
- [X] T011 [P] Refine badge size and padding to occupy at least 5% of image area as per SC-001

## Dependencies

- Phase 1 must be complete before Phase 4.
- Phase 2 must be complete before Phase 3.
- [US1] is a prerequisite for [US2].

## Parallel Execution Examples

- **Setup**: T003 (Dependencies) can be checked while T001/T002 (Resources) are being handled.
- **Foundational**: T004 (Colors) and T005 (Calculation) can be developed in parallel.
- **Validation**: T010 can be prepared while implementation is ongoing.
