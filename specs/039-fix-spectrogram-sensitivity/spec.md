# Feature Specification: Fix Spectrogram Sensitivity to Match rsudp

**Feature Branch**: `039-fix-spectrogram-sensitivity`
**Created**: 2026-02-12
**Status**: Draft
**Input**: User description: "spectrogramですが、rsudpと見え方が大きく異なります。ピークの位置とかは変わらないのですが、ピーク以外の部分も敏感に反応しているように見えます。感度や係数が大きく異なっていないか、references/rsudpの実装や、obspy, matplotlibと比較して、乖離している部分があれば修正しましょう。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Spectrogram visual parity with rsudp (Priority: P1)

A seismologist using the WebUI observes a spectrogram display that closely matches the visual output of rsudp's Python-based spectrogram. Non-peak frequency areas appear as dark/quiet background, while only genuine seismic signals produce bright regions. The overall contrast and sensitivity matches what they are accustomed to seeing in rsudp.

**Why this priority**: The spectrogram is a primary analysis tool for seismologists. If it displays excessive sensitivity (non-peak areas appearing bright), it obscures real signals and reduces the tool's diagnostic value. Matching rsudp's proven visual output is essential for user trust and correctness.

**Independent Test**: Can be tested by running both rsudp and WebUI side-by-side with the same seismic data stream and visually comparing spectrograms. Non-peak areas should appear similarly dark in both displays.

**Acceptance Scenarios**:

1. **Given** the WebUI is displaying a spectrogram during quiet seismic conditions, **When** the user compares with rsudp's spectrogram for the same channel and time window, **Then** the background (non-peak) areas have similar darkness/brightness levels.
2. **Given** a seismic event occurs, **When** the spectrogram shows a peak in both WebUI and rsudp, **Then** the peak stands out with similar contrast against the background in both systems.
3. **Given** the spectrogram is running, **When** no significant seismic activity is occurring, **Then** most of the spectrogram area appears dark (low values), not uniformly medium-bright.

---

### User Story 2 - Consistent spectrogram between backfill and live data (Priority: P2)

When a user connects to the WebUI, the backfill spectrogram (historical data) and subsequent live spectrogram data should have a seamless visual transition with consistent brightness and contrast levels.

**Why this priority**: A jarring visual difference between backfill and live data undermines trust in the display and can cause misinterpretation of seismic activity near the transition boundary.

**Independent Test**: Can be tested by opening the WebUI and observing the spectrogram at the point where backfill ends and live data begins. The contrast and brightness should not exhibit a sudden jump or drop.

**Acceptance Scenarios**:

1. **Given** the WebUI is opened and backfill data is displayed, **When** live streaming data begins arriving, **Then** there is no visible discontinuity in spectrogram brightness at the transition point.

---

### Edge Cases

- What happens when the signal is pure silence (zero amplitude)? Spectrogram should display as uniformly dark.
- How does the system handle extremely loud events that saturate the spectrogram? Peak regions should clip to maximum brightness without making quiet periods afterwards appear completely black.
- What happens with very low sample rates (e.g., 1 Hz) where NFFT is very small? The spectrogram should still produce meaningful visual output.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The spectrogram computation MUST normalize FFT output as Power Spectral Density (PSD), consistent with matplotlib's `specgram()` default behavior (dividing by window energy and frequency resolution).
- **FR-002**: The spectrogram MUST apply logarithmic (dB) scaling to PSD values before visual compression, matching matplotlib's default `scale='dB'` behavior.
- **FR-003**: The visual compression (power-law) MUST be applied to dB-scaled values, matching rsudp's `sg ** (1/10)` behavior where `sg` contains dB-scale data.
- **FR-004**: The spectrogram normalization MUST produce a dynamic range where quiet background appears dark (low u8 values ~0-80) and only genuine signals produce bright regions (high u8 values ~180-255).
- **FR-005**: Both backfill spectrogram computation and live incremental spectrogram computation MUST use the same scaling and normalization pipeline.
- **FR-006**: The mean subtraction (DC removal) MUST be applied before windowing, consistent with current behavior and rsudp's `data - mean` approach.

### Key Entities

- **PSD (Power Spectral Density)**: The frequency-domain representation of signal power, normalized by window energy and sampling rate. Units: power/Hz.
- **Running Maximum**: A per-channel tracking value used for dynamic normalization of live spectrogram data, with exponential decay to adapt to changing signal levels.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Side-by-side comparison of WebUI and rsudp spectrograms for the same seismic data shows visually similar contrast (non-peak background areas within 20% brightness of each other on a 0-255 scale).
- **SC-002**: During quiet conditions, at least 70% of spectrogram pixels have values below 80 (on a 0-255 scale), indicating proper background suppression.
- **SC-003**: Seismic event peaks in the spectrogram are at least 3x brighter than the surrounding background, ensuring clear signal visibility.
- **SC-004**: No visible discontinuity between backfill and live spectrogram data at the transition point.

## Assumptions

- The Hanning window and NFFT=128, HOP=13 parameters remain unchanged (matching current static values).
- The `inferno` colormap used on the frontend is consistent with rsudp and does not need modification.
- matplotlib's `specgram()` with default `mode='psd'` and `scale='dB'` is the target reference behavior.
- rsudp's `sg ** (1/10)` post-processing operates on dB-scale values returned by matplotlib, not on raw linear magnitude.

## Research Summary

Investigation of both implementations revealed the following root causes for the sensitivity difference:

| Aspect              | rsudp (matplotlib)           | Rust WebUI (current)        | Impact                                       |
| ------------------- | ---------------------------- | --------------------------- | -------------------------------------------- |
| FFT mode            | PSD (Power Spectral Density) | Raw magnitude squared       | Rust values are 10-100x larger               |
| Normalization       | By NFFT, window energy, Fs   | None (only running max)     | Rust is more sensitive to small changes       |
| dB scaling          | Yes (10*log10)               | No                          | Rust has ~100x less dynamic range compression |
| Power law target    | Applied to dB values         | Applied to linear mag sq    | Rust 0.1 power compresses less effectively    |
| Result              | Non-peak areas: ~50-100/255  | Non-peak areas: ~150-200/255| Rust background is 2-4x brighter             |
