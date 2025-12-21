# Research: Comprehensive Alerting System

## Decisions & Rationale

### 1. Waveform PNG Generation
- **Decision**: Use the `plotters` crate.
- **Rationale**: `plotters` is the standard library for data visualization in Rust. It supports drawing to a variety of backends, including bitmapped images (PNG). It is capable of handling the time-series data typical of seismic waveforms efficiently.
- **Alternatives considered**: `rust-gnuplot` (requires gnuplot installed), `charts` (more oriented towards high-level charts, less flexible for raw signal data).

### 2. Email (SMTP) Delivery
- **Decision**: Use the `lettre` crate.
- **Rationale**: `lettre` is the most mature and widely used SMTP client library in the Rust ecosystem. It supports asynchronous operation via `tokio`, which aligns with the project's async runtime.
- **Alternatives considered**: `mail-send` (newer, less proven in large-scale deployments).

### 3. Browser Audio Notification
- **Decision**: Use the standard HTML5 `Audio` API within the Next.js frontend.
- **Rationale**: For a simple alert sound, the standard browser API is sufficient and requires no external dependencies. The sound file will be served as a static asset.
- **Alternatives considered**: `howler.js` (unnecessary complexity for a single notification sound).

### 4. Asset Storage & Serving (PNGs)
- **Decision**: Store PNGs in a local `alerts/` directory within the `rsudp-rust` data directory. Serve them using `tower_http::services::ServeDir` via Axum.
- **Rationale**: Local storage is simple and meets the requirement. `ServeDir` integrates seamlessly with Axum to provide high-performance static file serving to the frontend.
- **Alternatives considered**: Cloud storage (overkill for this phase), embedding images in emails as CID (complementary, but doesn't solve WebUI display).

## Best Practices Found

### SMTP Async Sending
- Always use a connection pool or reuse the transport if sending multiple emails in a short burst to avoid handshake overhead.

### Large Waveform Plotting
- When plotting 60 seconds of 100Hz data (6000 points), downsampling for the PNG preview can improve performance and reduce file size without losing significant visual context.
