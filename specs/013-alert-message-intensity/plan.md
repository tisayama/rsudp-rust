# Implementation Plan: Intensity Inclusion in Alert Messages

**Branch**: `013-alert-message-intensity` | **Date**: 2025-12-21 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/013-alert-message-intensity/spec.md`

## Summary
Incorporate dynamic Japanese intensity descriptions into alert notifications. When an alert resets, the system will calculate the final Shindo level and append a formatted message (e.g., "震度 6弱相当の揺れを検出しました") to the history event, WebSocket broadcast, and email summary.

## Technical Context

**Language/Version**: Rust 1.7x (Backend)
**Primary Dependencies**: `rsudp-rust` internal modules (trigger, pipeline, alerts)
**Storage**: In-memory (24h alert history)
**Testing**: `cargo test`, visual check of logs and WebUI
**Target Platform**: Linux (Server)
**Project Type**: Library Enhancement
**Performance Goals**: Instant message formatting (<1ms overhead)
**Constraints**: Japanese character support in logs and WebUI

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Stability**: ✅ Logic is pure string formatting based on proven intensity values.
- **II. Rigorous Testing**: ✅ Will test all Shindo classes (0 to 7) against expected strings.
- **III. High Performance**: ✅ Zero impact on throughput.
- **IV. Clarity**: ✅ Reuse existing `get_shindo_class` mappings.
- **V. 日本語仕様**: ✅ Specification is in Japanese.
- **VI. 標準技術スタック**: ✅ Enhances existing Rust/WebUI interaction.

## Project Structure

### Documentation (this feature)

```text
specs/013-alert-message-intensity/
├── plan.md              # This file
├── research.md          # Phrasing decisions
├── data-model.md        # Updated AlertEvent structure
└── quickstart.md        # Verification steps
```

### Source Code (repository root)

```text
rsudp-rust/
├── src/
│   ├── web/
│   │   ├── alerts.rs    # Modified: Helper for message formatting
│   │   └── stream.rs    # Modified: Update WsMessage variant
│   └── pipeline.rs      # Modified: Logic to populate message on Reset
```

**Structure Decision**: Logic is integrated directly into the `pipeline` loop and `alerts` module to leverage the existing notification flow.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | | |