# Implementation Plan: Pure Rust MiniSEED Ingestion

**Branch**: `005-pure-rust-mseed` | **Date**: 2025-12-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-pure-rust-mseed/spec.md`

## Summary

Replace the C-based `mseed` crate with a native Rust implementation. This involves parsing the MiniSEED 2 Fixed Section Data Header and implementing a Steim2 decompression algorithm from scratch (or using a minimal pure-rust helper if identified during Phase 0).

## Technical Context

**Language/Version**: Rust 1.7x (latest stable)
**Primary Dependencies**: `byteorder` (endian-aware parsing), `chrono` (time handling), `thiserror` (error management).
**Storage**: N/A (Streaming parser)
**Testing**: Binary-level comparison with `obspy` / `004` output using `fdsnws.mseed`.
**Target Platform**: Linux (Initial), Platform-agnostic (Pure Rust advantage).
**Project Type**: Library module within `rsudp-rust`.
**Performance Goals**: Latency comparable to `libmseed`.
**Constraints**: No C toolchain required for build.
**Scale/Scope**: Support for MiniSEED 2 Data Records with Steim1/2 and integer encodings.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. 安定性と信頼性**: ✅ Pure Rust implementation eliminates FFI-related memory safety risks.
- **II. 厳密なテスト**: ✅ Direct comparison with the previous implementation ensures zero regression in data accuracy.
- **III. 高いパフォーマンス**: ✅ Zero-copy parsing and efficient bit-manipulation for Steim2 will be prioritized.
- **IV. コードの明瞭性と保守性**: ✅ Decoupling the bit-level decoder from the header parser ensures high maintainability.
- **V. 日本語による仕様策定**: ✅ This plan follows the Japanese-led specification process.

**Result**: All principles compliant.

## Project Structure

### Documentation (this feature)

```text
specs/005-pure-rust-mseed/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
└── quickstart.md        # Phase 1 output
```

### Source Code (repository root)

```text
rsudp-rust/
├── Cargo.toml           # Remove mseed, add byteorder
├── src/
│   ├── parser/
│   │   ├── mod.rs       # PureRustParser entry point
│   │   ├── header.rs    # MiniSEED 2 Header parsing
│   │   └── steim.rs     # Steim1/2 decompression logic
│   └── lib.rs           # Expose new parser
```

**Structure Decision**: Modularizing the parser into `header` and `steim` components to allow independent testing of the decompression logic.

## Complexity Tracking

N/A - No violations.