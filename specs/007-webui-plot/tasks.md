# Tasks: WebUI Plot System (Extended Polish)

**Input**: Design documents from `/specs/007-webui-plot/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/openapi.yaml, quickstart.md

**Organization**: Tasks are focused on final verification, robustness, and performance profiling.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 7: Extended Polish & Robustness

**Purpose**: Ensuring enterprise-grade reliability and performance observability

- [x] T029 [P] Implement `RingBuffer.test.ts` to verify circular logic and edge cases in `webui/lib/RingBuffer.test.ts`
- [x] T030 Implement a mock data producer in Rust to stress test the WebSocket broadcast with 10+ concurrent clients in `rsudp-rust/src/web/test_utils.rs`
- [x] T031 [P] Add FPS counter overlay to the frontend (debug mode) to monitor rendering performance in `webui/components/PerformanceMonitor.tsx`
- [x] T032 [P] Implement graceful shutdown for the Axum server to ensure WebSocket connections are closed cleanly in `rsudp-rust/src/main.rs`
- [x] T033 Add comprehensive error boundaries to the Next.js app to handle rendering failures gracefully in `webui/src/app/error.tsx`
- [x] T034 [P] Optimize binary payload: verify if using `f16` or quantized `i16` for samples reduces bandwidth significantly without losing precision in `rsudp-rust/src/web/mod.rs`
- [x] T035 Create a `Dockerfile` for the multi-project setup (Rust backend + Next.js frontend) in `Dockerfile`
- [x] T036 Final project consistency check: ensure all `tracing::info` and `console.log` follow a unified format across Rust and JS.

## Dependencies & Execution Order

- Phase 7 tasks are independent and can be executed in any order.
- T035 depends on T027 (Quickstart) being accurate for production environments.

## Implementation Strategy

1. **Verify Core Logic**: T029 ensures the data structure driving the UI is bulletproof.
2. **Stress Testing**: T030 and T031 verify the SC-002 (6+ channels at 60 FPS) under load.
3. **Production Readiness**: T032, T033, and T035 prepare the system for deployment beyond `localhost`.