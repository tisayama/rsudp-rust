# Implementation Plan: RSAM Calculation and UDP Forwarding

**Branch**: `035-rsam-calculation` | **Date**: 2026-02-10 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/035-rsam-calculation/spec.md`

## Summary

Implement RSAM (Real-time Seismic Amplitude Measurement) module that calculates mean, median, min, and max of absolute seismic amplitude values over a configurable time window, optionally applies deconvolution (sensitivity conversion), and periodically sends results via UDP in LITE/JSON/CSV formats. Follows the established Forward module pattern as architectural reference.

## Technical Context

**Language/Version**: Rust 1.7x (latest stable) + Edition 2021
**Primary Dependencies**: `tokio` (async runtime, net, time), `tracing` (logging), `serde` (serialization) — all already in Cargo.toml
**Storage**: In-memory sliding window buffer (Vec<f64> with time-based eviction)
**Testing**: `cargo test` with `tokio::test` for async tests
**Target Platform**: Linux server (same as existing rsudp-rust)
**Project Type**: Single project — `rsudp-rust/`
**Performance Goals**: Non-blocking pipeline integration; RSAM calculation <1ms for 10s window at 100Hz (1000 samples)
**Constraints**: Must not block or slow the main pipeline processing loop
**Scale/Scope**: Single channel monitoring, single destination UDP forwarding, ~1000 samples per calculation window

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. 安定性と信頼性 | PASS | Graceful error handling (FR-010, FR-015), fallback for missing sensitivity, non-blocking pipeline integration |
| II. 厳密なテスト | PASS | Unit tests for calculation, integration tests for UDP delivery, E2E tests (US4), channel filtering tests |
| III. 高いパフォーマンス | PASS | Non-blocking design, in-memory buffer, no allocations in hot path beyond sample accumulation |
| IV. コードの明瞭性と保守性 | PASS | Follows established Forward module pattern, single new file (rsam.rs), reuses existing channel matching |
| V. 日本語による仕様策定 | PASS | Spec and clarifications conducted in Japanese |
| VI. 標準技術スタック | N/A | No WebUI component in this feature |
| VII. 自己検証の義務 | PASS | All tests must pass before commit; E2E verification planned |
| VIII. ブランチ運用 | PASS | Working on feature branch `035-rsam-calculation` |

## Project Structure

### Documentation (this feature)

```text
specs/035-rsam-calculation/
├── plan.md              # This file
├── research.md          # Phase 0: design decisions
├── data-model.md        # Phase 1: entity definitions
├── quickstart.md        # Phase 1: usage guide
├── contracts/           # Phase 1: module API contract
│   └── rsam-module.md
└── tasks.md             # Phase 2: implementation tasks
```

### Source Code (repository root)

```text
rsudp-rust/
├── src/
│   ├── rsam.rs          # NEW: RSAM module (RsamManager, calculation, UDP sending)
│   ├── lib.rs           # MODIFY: add `pub mod rsam;`
│   ├── main.rs          # MODIFY: initialize RsamManager, pass to pipeline
│   └── pipeline.rs      # MODIFY: add rsam_manager parameter, feed segments to RSAM
└── tests/
    └── test_rsam.rs     # NEW: unit + integration + E2E tests
```

**Structure Decision**: Single project at `rsudp-rust/`. New module `rsam.rs` follows the same pattern as `forward.rs`. Existing files modified: `lib.rs`, `main.rs`, `pipeline.rs`. All tests in single test file `test_rsam.rs`.
