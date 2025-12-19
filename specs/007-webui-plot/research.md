# Research: WebUI Plot System

## Decision: Frontend Rendering Technology
- **Decision**: Use HTML5 Canvas API with `requestAnimationFrame` for waveform rendering.
- **Rationale**: Seismic data at 100Hz requires smooth scrolling and high-frequency updates. SVG or DOM-based approaches (like standard React components) would suffer from performance bottlenecks when handling thousands of points across multiple channels. Canvas allows direct pixel manipulation and is significantly more efficient for real-time time-series data.
- **Implementation Note**: Use React `ref` to manage the Canvas context directly, bypassing React's reconciliation cycle for the drawing loop to maintain 60 FPS.

## Decision: Data Transfer Format
- **Decision**: Use Binary WebSockets (Typed Arrays in JS) for seismic samples.
- **Rationale**: Sending seismic samples as JSON strings introduces significant overhead in both serialization (Rust) and parsing (JS). Binary transfer reduces bandwidth usage and CPU load, which is critical for supporting multiple concurrent clients and high-frequency data (100Hz+).
- **Alternatives Considered**: JSON (rejected due to overhead), MessagePack (considered but simple binary arrays are sufficient for raw samples).

## Decision: Backend WebSocket Architecture
- **Decision**: Use `axum` with `tokio::sync::broadcast` for data distribution.
- **Rationale**: Axum is the standard lightweight web framework for the Rust ecosystem. A broadcast channel allows the seismic data ingestion pipeline to produce samples once and efficiently distribute them to all connected WebSocket clients without duplicating processing logic.

## Decision: State Management & Synchronization
- **Decision**: Use a ring buffer (circular buffer) on the frontend to store incoming samples for the current view window.
## Decision: Binary Payload Optimization
- **Decision**: Retain `f32` for seismic samples.
- **Rationale**: While `i16` or `f16` would reduce bandwidth by 50%, `f32` provides sufficient precision for all seismic data types (counts and physical units) without quantization artifacts. Given the 100Hz sample rate and current throughput (~12 KB/s per client), the current bandwidth usage is well within acceptable limits for a real-time web dashboard.
