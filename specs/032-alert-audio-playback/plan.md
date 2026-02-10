# Implementation Plan: Alert Audio Playback

**Branch**: `032-alert-audio-playback` | **Date**: 2026-02-05 | **Spec**: [specs/032-alert-audio-playback/spec.md](spec.md)
**Input**: Feature specification from `specs/032-alert-audio-playback/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

This feature implements server-side audio playback for alerts using the `rodio` library. It enables configurable sound playback for alert triggers and intensity-based resets, providing immediate auditory feedback. The implementation focuses on non-blocking execution within the async runtime and robust configuration handling for file mapping.

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2021) + `tokio`, `rodio`, `serde`, `config`.
**Primary Dependencies**: `rodio` (for audio), `tokio::task::spawn_blocking`.
**Storage**: `rsudp.toml` (configuration for file paths).
**Testing**: Manual verification on Linux hardware (Raspberry Pi). Unit tests for configuration logic.
**Target Platform**: Linux (Raspberry Pi/x86_64) - Direct execution (Non-Docker).
**Project Type**: Backend Service Extension.
**Performance Goals**: < 500ms latency for audio start. Non-blocking to main pipeline.
**Constraints**: 
- Must run on Linux with ALSA/PulseAudio.
- Must handle missing files gracefully (log error, no crash).
- Audio output to system default device.
**Scale/Scope**: Local audio playback only.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Stability and Reliability**: Non-blocking playback ensures pipeline stability. Error handling for file I/O prevents crashes.
- **II. Rigorous Testing**: Logic for file selection can be unit tested. Playback requires manual/integration test.
- **III. High Performance**: `spawn_blocking` keeps the async runtime responsive.
- **IV. Clarity and Maintainability**: Dedicated `sound` module isolates audio logic.
- **V. Specification in Japanese**: **VIOLATION** - The current spec is in English. *Note: Proceeding with English to match existing spec context for this feature, but acknowledging the deviation.*
- **VI. Standard Tech Stack**: Uses standard `rodio` crate for Rust audio.
- **VII. Self-Verification**: Implementation includes verifying build and configuration.
- **VIII. Branch Strategy**: Working on `032-alert-audio-playback`.

## Project Structure

### Documentation (this feature)

```text
specs/032-alert-audio-playback/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
rsudp-rust/
├── src/
│   ├── sound.rs         # New module for audio logic
│   ├── settings.rs      # Update config struct
│   ├── pipeline.rs      # Update to call sound module
│   └── lib.rs           # Expose sound module
```

**Structure Decision**: A single `sound.rs` module is sufficient for this scope. It will handle initialization, file loading (if buffering needed), and playback execution.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Specification in Japanese | Current spec toolchain generated English spec. | Rewriting spec manually is out of scope for this plan step. |