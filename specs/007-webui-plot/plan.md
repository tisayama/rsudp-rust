# Implementation Plan: WebUI Plot System

**Branch**: `007-webui-plot` | **Date**: 2025-12-19 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/007-webui-plot/spec.md`

## Summary

Build a high-performance WebUI for real-time seismic waveform visualization. The system comprises a Rust backend using Axum for REST and WebSockets, and a Next.js frontend using the HTML5 Canvas API for efficient rendering. This replaces the legacy X11 dependency with a modern, responsive web dashboard.

## Technical Context

**Language/Version**: Rust 1.7x (Backend), TypeScript / Next.js 14+ (Frontend)
**Primary Dependencies**: 
- Backend: `axum` (web framework), `tokio` (runtime), `serde` (serialization), `tower-http` (CORS/static files).
- Frontend: `Next.js`, `Tailwind CSS`, `Lucide React` (icons), `Canvas API`.
**Storage**: In-memory ring buffer for real-time sample streaming; JSON file or local storage for persistent UI settings.
**Testing**: `cargo test` for streaming logic; `Jest` + `Playwright` for frontend rendering and integration.
**Target Platform**: Web Browser (Desktop/Tablet)
**Project Type**: Web application (frontend + backend)
**Performance Goals**: 60 FPS rendering for 6+ channels; WebSocket latency < 100ms.
**Constraints**: No X11/Tkinter dependencies; must adhere to constitutional tech stack (Next.js, Tailwind, Rust REST/WS).
**Scale/Scope**: Supports multiple concurrent browser clients viewing real-time streams.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Stability)**: WebSocket reconnection logic and data buffering on frontend ensure stability during network jitter.
- **Principle II (Rigorous Testing)**: Integration tests for the full WS-to-Canvas pipeline are planned.
- **Principle III (Performance)**: Canvas rendering and binary WebSocket packets minimize CPU/bandwidth usage.
- **Principle IV (Clarity)**: Component-based Next.js structure and modular Rust handlers.
- **Principle VI (Standard Tech Stack)**: Uses Next.js, Tailwind, Rust REST API, and WebSockets as required.

## Project Structure

### Documentation (this feature)

```text
specs/007-webui-plot/
├── plan.md              # This file
├── research.md          # Rendering and streaming decisions
├── data-model.md        # Waveform packet and settings structures
├── quickstart.md        # Setup instructions
├── contracts/           # OpenAPI and WS message definitions
└── tasks.md             # Implementation tasks (Phase 2)
```

### Source Code

```text
rsudp-rust/              # Backend
├── src/
│   ├── web/             # Axum handlers and WS logic
│   │   ├── mod.rs
│   │   ├── routes.rs    # REST endpoints
│   │   └── stream.rs    # WebSocket broadcast logic
│   └── main.rs          # Entry point integration

webui/                   # Frontend (Next.js)
├── components/          # React components (Waveform, ControlPanel)
├── hooks/               # useWebSocket, useCanvas
├── lib/                 # RingBuffer, formatting utils
└── pages/               # Main dashboard
```

**Structure Decision**: Multi-project layout. Backend integrated into existing `rsudp-rust` crate (optional feature); Frontend in a new `webui/` directory at repository root.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
