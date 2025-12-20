# Implementation Plan: UDP MiniSEED Streamer for Simulation

**Branch**: `009-udp-mseed-streamer` | **Date**: 2025-12-19 | **Spec**: [specs/009-udp-mseed-streamer/spec.md](spec.md)
**Input**: Feature specification from `/specs/009-udp-mseed-streamer/spec.md`

## Summary
Implement a high-precision, low-memory UDP streamer that replays MiniSEED files in real-time or accelerated speeds. It indexes binary files to handle 2GB+ datasets and converts data into `rsudp`-compatible UDP packets to simulate live Raspberry Shake sensors.

## Technical Context

**Language/Version**: Rust 1.7x  
**Primary Dependencies**: `tokio` (async), `clap` (CLI), `tracing` (logging), `byteorder` (binary parsing), `chrono` (time), `serde_json` (packet format)  
**Storage**: N/A (Read-only filesystem access)  
**Testing**: `cargo test`, Integration test with `fdsnws.mseed`  
**Target Platform**: Linux / macOS / Windows  
**Project Type**: Single binary (within `rsudp-rust` workspace)  
**Performance Goals**: <5ms timing jitter, constant <50MB memory footprint  
**Constraints**: Absolute time-aware playback, strict sequential inter-channel interleaving  
**Scale/Scope**: Simulation tool for developers and QA  

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Stability**: ✅ Implemented with robust error handling for IO and Socket operations.
- **II. Rigorous Testing**: ✅ Includes replay verification tests comparing emitted timestamps.
- **III. High Performance**: ✅ Zero-copy file access where possible, low memory via indexing.
- **IV. Clarity**: ✅ Modular design separating Parser, Indexer, and Clock.
- **V. Japanese Spec**: ✅ Specification exists in Japanese.

## Project Structure

### Documentation (this feature)

```text
specs/009-udp-mseed-streamer/
├── plan.md              # This file
├── research.md          # Timing and Indexing strategy
├── data-model.md        # Index and Config entities
├── quickstart.md        # CLI usage guide
└── tasks.md             # Implementation steps
```

### Source Code (repository root)

```text
rsudp-rust/
├── src/
│   ├── bin/
│   │   └── streamer.rs  # Main entry point for the streamer CLI
│   └── parser/          # Reused Steim/MiniSEED logic
└── tests/
    └── integration_streamer.rs
```

**Structure Decision**: Added as a new binary target in `Cargo.toml` to reuse the existing `rsudp-rust` library logic (parsers) without bloating the main server binary.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Two-pass File Access | To support 2GB files with low memory | Single-pass load is limited by RAM. |
| Drift-correction Clock | To meet 5ms jitter SC-001 | Simple `sleep(duration)` accumulates OS scheduler lag. |