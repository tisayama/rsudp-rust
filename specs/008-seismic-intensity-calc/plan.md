# Implementation Plan: Japanese Seismic Intensity Calculation

**Branch**: `008-seismic-intensity-calc` | **Date**: 2025-12-19 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/008-seismic-intensity-calc/spec.md`

## Summary

Implement the JMA standard Measured Seismic Intensity calculation. This module will process 3-component acceleration data (ENE, ENN, ENZ) through a frequency-domain filtering pipeline (FFT -> JMA Filter -> IFFT), calculate the vector sum, and determine the 0.3s-cumulative peak acceleration `a`. Results will be broadcast to the WebUI dashboard in real-time.

## Technical Context

**Language/Version**: Rust 1.7x
**Primary Dependencies**: `rustfft` (Frequency domain filtering), `chrono` (Time handling), `tokio::sync::broadcast` (Distribution to WebUI), `serde` (Serialization).
**Storage**: In-memory sliding window (RingBuffer) of 60 seconds.
**Testing**: `cargo test` with comparison against manual calculation and reference data `fdsnws.mseed`.
**Target Platform**: Linux / Cross-platform (Standard Rust)
**Project Type**: Single project component (Library + CLI extension).
**Performance Goals**: < 100ms calculation time for a 60s window (6000 samples) every 1s.
**Constraints**: Must match JMA filter response exactly. Input must be 3 synchronized components.
**Scale/Scope**: Real-time monitoring for a single station.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Stability)**: Frequency domain calculation is stable and well-understood for this application.
- **Principle II (Rigorous Testing)**: SC-001 mandates accuracy verification against known reference data.
- **Principle III (Performance)**: FFT-based filtering is O(N log N) and highly efficient for 6000-sample windows.
- **Principle IV (Clarity)**: Separation of intensity calculation logic from data ingestion and UI broadcast.
- **Principle V (Japanese Spec)**: Specifications and clarifications were developed in Japanese.

## Project Structure

### Documentation (this feature)

```text
specs/008-seismic-intensity-calc/
├── plan.md              # This file
├── research.md          # JMA filter and algorithm research
├── data-model.md        # IntensityConfig and IntensityResult
├── quickstart.md        # Usage examples
├── contracts/           # WebSocket message schemas
└── tasks.md             # Execution steps (Phase 2)
```

### Source Code

```text
rsudp-rust/
├── src/
│   ├── intensity/       # New module for intensity calculation
│   │   ├── mod.rs       # Public API
│   │   ├── filter.rs    # JMA digital filter coefficients/logic
│   │   └── calc.rs      # Intensity value and class determination
│   └── main.rs          # Integration with CLI flags and pipeline
└── tests/
    └── integration_intensity.rs # Verification with fdsnws.mseed
```

**Structure Decision**: Create a dedicated `src/intensity/` module within `rsudp-rust` to keep the logic isolated and testable.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
