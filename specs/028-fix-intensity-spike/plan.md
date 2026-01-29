# Implementation Plan - Fix Max Intensity Spike in Noise Data

This plan outlines the steps to investigate, reproduce, and fix the intensity calculation spike observed in real-world noise data, ensuring adherence to the JMA standard and Rust project conventions.

## User Review Required

> [!IMPORTANT]
> **Critical items requiring user attention before proceeding:**
>
> 1.  **Dataset Availability**: Confirm the 60-minute dataset (`normal.mseed`) is available in `references/mseed/`.
> 2.  **Reproduction**: The spike is reported around T00:52:00. Note: Data starts at T00:02:00, so this is approximately 3000 seconds (50 minutes) into the recording.
> 3.  **Scope**: This fix focuses on the `IntensityManager` logic and potentially the `TriggerManager` filter initialization if shared.

## Technical Context

### Architecture & Dependencies

-   **Component**: `rsudp-rust/src/intensity.rs` (IntensityManager), `rsudp-rust/src/trigger.rs` (Filter/Trigger logic).
-   **Dependencies**: `rustfft` (used for frequency domain filtering in intensity calc), `biquad` (used in trigger, maybe intensity?).
-   **Data Flow**: MiniSEED Stream -> Parser -> Pipeline -> IntensityManager -> Calculation -> Log/Broadcast.

### Technology Selection

-   **Language**: Rust 1.7x (stable).
-   **Libraries**: Standard library, `rustfft`, internal modules.
-   **Testing**: Integration test using the `normal.mseed` file.

### Constitution Check

-   **I. Stability and Reliability**: Fix addresses a critical false positive issue (reliability). "Short-term fixes" are discouraged; we must find the *theoretical* cause.
-   **II. Rigorous Testing**: Mandatory testing with the provided real-world dataset.
-   **III. High Performance**: Fix must not degrade pipeline throughput.
-   **IV. Clarity**: Document the fix clearly in comments.
-   **V. Japanese Spec**: Specifications are in Japanese/English mixed.
-   **VII. Self-Verification**: Must verify fix before merge.
-   **VIII. Branching**: Working on `028-fix-intensity-spike`.

## Phase 0: Research & Design

### 0.1. Investigation Tasks

-   [ ] **Analyze `IntensityManager` Implementation**: Review `src/intensity.rs` to understand the current JMA calculation logic.
-   [ ] **Analyze Data**: Inspect `references/mseed/normal.mseed` for data gaps, overlaps, or DC offsets near T00:52:00 (+3000s).
-   [ ] **Reproduce**: Create a test binary `verify_intensity_spike` that reads the file and outputs intensity every second, scanning the 50-minute mark.

### 0.2. Design Decisions (Preliminary)

-   **Hypothesis 1 (Filter Ringing)**: FFT-based filtering on a buffer with a DC offset or trend (without detrending/tapering) causes edge effects.
-   **Hypothesis 2 (Buffer Management)**: Ring buffer wrap-around or initialization glitch.
-   **Hypothesis 3 (Gap Handling)**: A data gap filled with zeros creates a step function.

**Decision**: We will prioritize **Hypothesis 1 & 3** (DC offset/Gap + Filter response).

### 0.3. Technical Standards

-   **Filter**: JMA standard requires specific bandpass/highpass. Ensure `rustfft` implementation matches the standard.
-   **Windowing**: JMA uses a specific window length. Ensure we aren't processing too short a segment.

## Phase 1: Implementation Steps

### 1.1. Reproduction Tool

-   **Task**: Create `rsudp-rust/src/bin/repro_spike.rs`.
    -   Read `normal.mseed`.
    -   Feed it into `IntensityManager` exactly as the pipeline does.
    -   Print `timestamp, intensity`.
    -   Assert failure if intensity > 2.0 around 3000s-3200s.

### 1.2. Fix Implementation

-   **Task**: Modify `src/intensity.rs`.
    -   **Fix**: Implement `detrend` (linear/constant) and `taper` (cosine/hanning) before FFT.
    -   **Fix**: Check for `NaN` / `Inf` handling.
    -   **Fix**: Improve gap handling (if applicable).

### 1.3. Verification

-   **Task**: Run `repro_spike.rs` with the fix.
    -   Confirm max intensity is low (e.g., < 1.0) throughout the spike window.
-   **Task**: Run standard `cargo test`.

## Phase 2: Validation & Documentation

### 2.1. Integrated Test

-   **Task**: Run the full `rsudp-rust` pipeline with the file using `streamer`.
-   **Task**: Verify logs show stable intensity.

### 2.2. Documentation

-   **Task**: Update `GEMINI.md` with the fix details.

## Gate Checks

-   [ ] **Reproduction Confirmed**: Can we see the spike around +3000s?
-   [ ] **Fix Verified**: Does the spike disappear?
-   [ ] **Side Effects**: Does valid earthquake data still produce correct intensity? (Regression test with `fdsnws.mseed`).

## Artifacts

-   `rsudp-rust/src/bin/repro_spike.rs`
-   `rsudp-rust/src/intensity.rs` (Modified)
-   `logs/repro_before.log`
-   `logs/repro_after.log`