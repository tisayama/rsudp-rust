# Implementation Plan: STA/LTA Alert System

**Branch**: `006-sta-lta-alert` | **Date**: 2025-12-19 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/006-sta-lta-alert/spec.md`

## Summary

Implement a real-time STA/LTA alert system in Rust that achieves functional parity with `rsudp`. The system will process incoming seismic data samples, apply optional filtering, calculate the recursive STA/LTA ratio, and trigger alarm/reset events based on configurable thresholds.

## Technical Context

**Language/Version**: Rust 1.7x
**Primary Dependencies**: `tokio` (async runtime), `chrono` (time handling), `thiserror` (error handling), `byteorder` (parsing). Verification requires Python 3.x + `obspy`.
**Storage**: In-memory state for recursive averages and trigger status.
**Testing**: `cargo test` for unit/integration tests; Python scripts for cross-validation with ObsPy results.
**Target Platform**: Linux (Standard server environments)
**Project Type**: Single project (Library/Binary component within `rsudp-rust`)
**Performance Goals**: Alarm latency < 200ms, CPU usage < 5% of a single core for a 100Hz stream.
**Constraints**: Must handle data gaps by resetting state; must handle out-of-order data by discarding it.
**Scale/Scope**: Single-channel monitoring per instance; scalable by deploying multiple instances.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Stability)**: Robust handling of data gaps and out-of-order samples is specified in the Edge Cases.
- **Principle II (Testing)**: Cross-validation with ObsPy is a core success criterion.
- **Principle III (Performance)**: Rust implementation targets low latency and minimal resource footprint.
- **Principle IV (Clarity)**: Following idiomatic Rust patterns (e.g., `thiserror`, `tokio`).
- **Principle V (Japanese Spec)**: Specification clarifications and requirements were defined in Japanese.

## Project Structure

### Documentation (this feature)

```text
specs/006-sta-lta-alert/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (to be generated)
```

### Source Code (repository root)

```text
rsudp-rust/
├── src/
│   ├── pipeline.rs      # Integration point for the alert system
│   ├── trigger.rs       # STA/LTA calculation and trigger logic
│   └── parser/          # Data parsing
└── tests/
    ├── integration_alert.rs
    └── scripts/
        └── verify_stalta.py
```

**Structure Decision**: Single project structure using existing `rsudp-rust` layout. Alert logic will reside in `src/trigger.rs`.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
