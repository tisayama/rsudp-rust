# Implementation Plan: rsudp準拠のアラート投稿タイミング実装 (rsudp-style Alert Post Timing)

**Branch**: `014-rsudp-alert-timing` | **Date**: 2025-12-21 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/014-rsudp-alert-timing/spec.md`

## Summary
Implement a timer-based notification trigger that mimics `rsudp`'s behavior. Instead of using the `RESET` event, the system will schedule a background task upon `TRIGGER` that waits for a specific ratio of the plot duration (default 70%) before generating the snapshot and sending the final summary notification.

## Technical Context

**Language/Version**: Rust 1.7x
**Primary Dependencies**: `tokio` (Timers, Tasks), `uuid`, `chrono`
**Storage**: N/A (In-memory state)
**Testing**: Integration test with simulated delay
**Target Platform**: Linux / Cross-platform
**Project Type**: Library Enhancement
**Performance Goals**: <10ms scheduling overhead, no blocking of the data loop
**Constraints**: Must match `rsudp` implementation logic for `save_pct`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Stability**: ✅ Background tasks via `tokio::spawn` ensure the data loop remains unblocked.
- **II. Rigorous Testing**: ✅ Will verify the 70% delay logic using 1x speed stream.
- **III. High Performance**: ✅ Minimal overhead for scheduling.
- **IV. Clarity**: ✅ Separation of detection (TriggerManager) from notification scheduling (Pipeline).
- **V. 日本語仕様**: ✅ Specification is in Japanese.

## Project Structure

### Documentation (this feature)

```text
specs/014-rsudp-alert-timing/
├── plan.md              # This file
├── research.md          # Timer logic and rsudp matching
├── data-model.md        # AlertTask entity
└── tasks.md             # Execution steps
```

### Source Code (repository root)

```text
rsudp-rust/
├── src/
│   ├── pipeline.rs      # Modified: Refactor notification trigger logic
│   └── web/
│       └── alerts.rs    # Modified: Support for windowed max intensity
```

**Structure Decision**: Refactor the `RESET` branch in `pipeline.rs` to remove notification logic and add `TRIGGER` branch logic to spawn the timer task.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Multi-channel Task Management | Correctness | Simple global timer fails when multiple stations/channels trigger concurrently. |