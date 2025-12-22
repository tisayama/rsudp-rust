# Tasks: rsudp Plot Timestamp & Grid Fix

**Feature Branch**: `015-fix-plot-timestamp`
**Implementation Strategy**: Refine the drawing logic in `plot.rs` to replace relative time offsets with UTC timestamps and adjust the grid visibility to match rsudp aesthetics.

## Phase 1: Setup

- [x] T001 Ensure `chrono` and `plotters` dependencies are up to date in `rsudp-rust/Cargo.toml`

## Phase 2: Foundational (Timing Logic)

- [x] T002 Implement `format_utc_timestamp` helper in `rsudp-rust/src/web/plot.rs` (converts f64 offset to HH:MM:SS string)

## Phase 3: [US1] UTC Timestamp on X-axis (Priority: P1)

**Goal**: Show absolute time on the plot's X-axis.
**Independent Test**: Generate a plot and verify the X-axis labels match the expected UTC time based on the data start time.

- [x] T003 [US1] Apply `x_label_formatter` to the Spectrogram chart in `rsudp-rust/src/web/plot.rs`
- [x] T004 [US1] Synchronize X-axis ranges and formatting between Waveform and Spectrogram areas in `rsudp-rust/src/web/plot.rs`
- [x] T005 [US1] Disable X-axis labels for the Waveform charts to reduce clutter in `rsudp-rust/src/web/plot.rs`

## Phase 4: [US2] Waveform Grid Visibility (Priority: P2)

**Goal**: Remove redundant grid lines from waveforms to match rsudp style.
**Independent Test**: Generate a plot and verify the waveform background is solid dark without grey grid lines.

- [x] T006 [US2] Use `.disable_mesh()` for the Waveform chart configuration in `rsudp-rust/src/web/plot.rs`
- [x] T007 [US2] Ensure Y-axis labels (Counts/Units) remain visible while the grid is hidden in `rsudp-rust/src/web/plot.rs`

## Phase 5: Polish & Validation

- [x] T008 [P] Update `rsudp-rust/tests/test_plot_comparison.rs` to verify the new visual style
- [x] T009 [P] Adjust X-axis label frequency (ticks) to prevent overlapping of timestamp strings

## Dependencies

- Phase 2 must be complete before Phase 3.
- US1 should be verified before finalizing US2 to ensure coordinate alignment remains correct.

## Parallel Execution Examples

- **Polish**: T008 and T009 can be developed in parallel once the core drawing logic is updated.
- **Verification**: Integration tests can be updated while documentation is being polished.
