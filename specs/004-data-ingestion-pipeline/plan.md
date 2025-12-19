# Implementation Plan: Data Ingestion Pipeline

**Branch**: `004-data-ingestion-pipeline` | **Date**: 2025-12-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-data-ingestion-pipeline/spec.md`

## Summary

Build a robust data ingestion pipeline that parses incoming byte streams (via UDP or from MiniSEED files) and feeds the resulting numerical samples into dynamically assigned `RecursiveStaLta` filters. The pipeline will support full MiniSEED parsing and handle data gaps gracefully.

## Technical Context

**Language/Version**: Rust 1.7x (latest stable)
**Primary Dependencies**: `tokio` (async runtime), `clap` (CLI), `tracing` (logging), `mseed` (potential for MiniSEED parsing)
**Storage**: In-memory (filter state management)
**Testing**: Integration tests using sample MiniSEED files; comparison with `obspy` results.
**Target Platform**: Linux (cross-platform compatible)
**Project Type**: Single binary application
**Performance Goals**: >1,000,000 samples/sec throughput in simulation mode.
**Constraints**: Full MiniSEED compliance (Steim encoding, headers).
**Scale/Scope**: Multi-channel data ingestion.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. 安定性と信頼性**: ✅ Comprehensive error handling for malformed packets and data gaps ensures system stability.
- **II. 厳密なテスト**: ✅ Simulation mode allows repeatable tests using MiniSEED files, verifiable against known references.
- **III. 高いパフォーマンス**: ✅ Async pipeline using Tokio channels minimizes ingestion latency.
- **IV. コードの明瞭性と保守性**: ✅ Decoupling parsing from calculation logic ensures a clean and maintainability architecture.
- **V. 日本語による仕様策定**: ✅ Specifications and clarifications handled in Japanese as per user request.

**Result**: All principles compliant.

## Project Structure

### Documentation (this feature)

```text
specs/004-data-ingestion-pipeline/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
rsudp-rust/
├── src/
│   ├── parser/          # New module for MiniSEED/Raw parsing
│   │   ├── mod.rs
│   │   └── mseed.rs
│   ├── pipeline.rs      # Pipeline orchestration logic
│   ├── main.rs          # CLI updated for simulation mode
│   └── trigger.rs       # Updated for NSLC-based state management
└── tests/
    └── data/            # Sample MiniSEED files for testing
```

**Structure Decision**: A dedicated `parser` module will be created. The pipeline logic will be separated from the network/calculation modules to maintain clear responsibilities.

## Complexity Tracking

N/A - No violations.