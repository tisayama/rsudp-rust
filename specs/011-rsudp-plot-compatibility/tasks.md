# Tasks: rsudp Plot Compatibility

**Feature Branch**: `011-rsudp-plot-compatibility`
**Implementation Strategy**: Replace the simple waveform plot with a high-fidelity `rsudp` clone including spectrograms.

## Phase 1: Setup

- [x] T001 Add `rustfft` and `colorous` (or similar) dependencies to `rsudp-rust/Cargo.toml`
- [x] T002 Create new module `rsudp-rust/src/web/plot.rs` and expose it in `rsudp-rust/src/web/mod.rs`

## Phase 2: Foundational (Spectrogram Calculation)

- [x] T003 Implement `compute_spectrogram` function in `plot.rs` using `rustfft` (NFFT=256, noverlap=128)
- [x] T004 Implement `viridis` colormap logic in `plot.rs` to map PSD values to RGB
- [x] T005 [P] Create unit tests for spectrogram calculation in `rsudp-rust/src/web/plot.rs`

## Phase 3: [US1] rsudp-style Waveform Plot (Priority: P1)

**Goal**: Match the visual style of rsudp waveforms.
**Independent Test**: Generate a waveform-only plot and compare it visually with an rsudp screenshot.

- [x] T006 [US1] Implement `draw_waveform` function in `plot.rs` with `plotters` (styling: grey grid, blue line, tight margins)
- [x] T007 [US1] Update `generate_snapshot` in `alerts.rs` to use `plot.rs` functions

## Phase 4: [US2] Spectrogram Rendering (Priority: P2)

**Goal**: Add the spectrogram panel below the waveform.
**Independent Test**: Generate a plot and verify the spectrogram appears and aligns with the waveform time axis.

- [x] T008 [US2] Implement `draw_spectrogram` function in `plot.rs` using `BitMapBackend`
- [x] T009 [US2] Create a composite layout function in `plot.rs` to stack Waveform (top) and Spectrogram (bottom)

## Phase 5: [US3] Multi-channel Vertical Stack (Priority: P3)

**Goal**: Combine multiple channels (EHZ, EHE, EHN) into a single vertical image.
**Independent Test**: Trigger a 3-channel alert and verify one tall PNG is generated containing all 3 plots.

- [x] T010 [US3] Refactor `generate_snapshot` to accept a `HashMap<Channel, Samples>` instead of single channel data
- [x] T011 [US3] Implement multi-panel vertical layout logic in `plot.rs` (3 channels * 2 subplots = 6 panels)
- [x] T012 [US3] Update pipeline to pass all channel data to `generate_snapshot` on alert reset

## Phase 6: Polish

- [x] T013 [P] Add metadata annotations (Station, Start Time, PGA) to the plot title
- [x] T014 [P] Optimize color mapping performance (lookup table)

## Dependencies

- Phase 2 (Foundational) is required for Phase 4.
- Phase 3 replaces the current plotting logic, so it blocks Phase 4 and 5 integration.

## Parallel Execution Examples

- **Foundational**: T003 (FFT logic) and T004 (Colormap) can be developed independently.
- **US3**: T010 (Pipeline data gathering) can be implemented while T011 (Plot layout) is being worked on.
