# Implementation Plan: Comprehensive Alerting System

**Branch**: `010-comprehensive-alerting` | **Date**: 2025-12-19 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/010-comprehensive-alerting/spec.md`

## Summary
Implement a multi-channel alerting system that provides real-time visual and audio notifications via Next.js WebUI, automated waveform snapshot generation (PNG), and tiered email notifications (Trigger & Reset) using Rust backend.

## Technical Context

**Language/Version**: Rust 1.7x (Backend), TypeScript / Next.js 14+ (Frontend)
**Primary Dependencies**: `plotters` (Plotting), `lettre` (SMTP), `tower-http` (Static serving), `axum` (REST API), `tokio` (Async runtime)
**Storage**: Local filesystem for PNGs, In-memory for 24h history (extendable to JSON/SQLite if persistence required)
**Testing**: `cargo test` for STA/LTA logic, `jest` for UI state transitions
**Target Platform**: Linux (Server), Modern Browsers
**Project Type**: Web Application
**Performance Goals**: <500ms notification latency, <10s snapshot generation
**Constraints**: Telegram integration is explicitly out of scope.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Stability**: ✅ Multi-threaded async processing ensures alerts don't block the data ingestion pipeline.
- **II. Rigorous Testing**: ✅ Integration tests will simulate earthquake arrivals to verify the full chain (Trigger -> Plot -> Email).
- **III. High Performance**: ✅ `plotters` provides efficient image rendering; image generation is offloaded to a separate task.
- **IV. Clarity**: ✅ Standard REST patterns for settings and history management.
- **V. Japanese Spec**: ✅ Specification exists in Japanese.
- **VI. Standard Tech Stack**: ✅ Next.js + Tailwind for UI, Rust for backend.

## Project Structure

### Documentation (this feature)

```text
specs/010-comprehensive-alerting/
├── plan.md              # This file
├── research.md          # Plotting and SMTP choices
├── data-model.md        # Alert entities and states
├── quickstart.md        # How to test alerts
├── contracts/           # API and WebSocket schemas
└── tasks.md             # Execution steps
```

### Source Code

```text
rsudp-rust/
├── src/
│   ├── web/
│   │   ├── alerts.rs    # Snapshot generation & SMTP logic
│   │   └── history.rs   # History management
│   └── static/          # Notification sound files
webui/
├── src/app/history/     # Alert history page
├── public/sounds/       # Alert WAV/MP3 files
└── hooks/useAlerts.ts   # UI side WebSocket handler
```

**Structure Decision**: Integrated into existing `rsudp-rust` and `webui` projects. Snapshot files served from a dedicated `alerts/` directory.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Two-pass Email (Trigger+Reset) | Requested in spec for accuracy | Reset-only email lacks immediate urgency. |
| In-memory History | Simple 24h window | DB persistence adds maintenance overhead for a dashboard utility. |