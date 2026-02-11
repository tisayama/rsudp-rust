# rustrsudp_speckit Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-12-19

## Active Technologies
- Rust 1.7x (latest stable) + `tokio` (async runtime, net), `clap` (CLI args), `tracing` (logging) (002-udp-receiver)
- In-memory queue (Tokio MPSC channel) (002-udp-receiver)
- Rust 1.7x (latest stable) + None (standard library only for calculation). Python 3.x + `obspy` required for verification tests. (003-sta-lta-calc)
- N/A (Processing logic only) (003-sta-lta-calc)
- Rust 1.7x (latest stable) + `tokio` (async runtime), `clap` (CLI), `tracing` (logging), `mseed` (potential for MiniSEED parsing) (004-data-ingestion-pipeline)
- In-memory (filter state management) (004-data-ingestion-pipeline)
- Rust 1.7x (latest stable) + `byteorder` (endian-aware parsing), `chrono` (time handling), `thiserror` (error management). (005-pure-rust-mseed)
- N/A (Streaming parser) (005-pure-rust-mseed)
- Rust 1.7x + `tokio` (async runtime), `chrono` (time handling), `thiserror` (error handling), `byteorder` (parsing). Verification requires Python 3.x + `obspy`. (006-sta-lta-alert)
- In-memory state for recursive averages and trigger status. (006-sta-lta-alert)
- Rust 1.7x (Backend), TypeScript / Next.js 14+ (Frontend) (007-webui-plot)
- In-memory ring buffer for real-time sample streaming; JSON file or local storage for persistent UI settings. (007-webui-plot)
- Rust 1.7x + `rustfft` (Frequency domain filtering), `chrono` (Time handling), `tokio::sync::broadcast` (Distribution to WebUI), `serde` (Serialization). (008-seismic-intensity-calc)
- In-memory sliding window (RingBuffer) of 60 seconds. (008-seismic-intensity-calc)
- Rust 1.7x + `tokio` (async), `clap` (CLI), `tracing` (logging), `byteorder` (binary parsing), `chrono` (time), `serde_json` (packet format) (009-udp-mseed-streamer)
- N/A (Read-only filesystem access) (009-udp-mseed-streamer)
- Rust 1.7x (Backend), TypeScript / Next.js 14+ (Frontend) + `plotters` (Plotting), `lettre` (SMTP), `tower-http` (Static serving), `axum` (REST API), `tokio` (Async runtime) (010-comprehensive-alerting)
- Local filesystem for PNGs, In-memory for 24h history (extendable to JSON/SQLite if persistence required) (010-comprehensive-alerting)
- Rust 1.7x + `plotters` (Plotting), `rustfft` (Spectrogram), `colorous` (Colormaps, if needed) (011-rsudp-plot-compatibility)
- Local filesystem (existing `alerts/` directory) (011-rsudp-plot-compatibility)
- Rust 1.7x + `plotters` (with `ab_glyph`), `rustfft`, `chrono` (012-intensity-on-plot)
- Embedded `NotoSansJP-Bold.ttf` (via `include_bytes!`) (012-intensity-on-plot)
- Rust 1.7x (Backend) + `rsudp-rust` internal modules (trigger, pipeline, alerts) (013-alert-message-intensity)
- In-memory (24h alert history) (013-alert-message-intensity)
- Rust 1.7x + `tokio` (Timers, Tasks), `uuid`, `chrono` (014-rsudp-alert-timing)
- N/A (In-memory state) (014-rsudp-alert-timing)
- Rust 1.7x + `plotters`, `chrono` (015-fix-plot-timestamp)
- Rust 1.7x (Edition 2024) + `serde` (serialization), `toml` (TOML parsing), `serde_yaml` (YAML parsing), `config` (configuration management), `clap` (CLI args) (016-add-rsudp-config)
- N/A (Configuration files on disk) (016-add-rsudp-config)
- Rust 1.7x (Edition 2024) + `serde_json` (will be replaced/bypassed for custom formatting), `chrono` (timestamps) (017-fix-streamer-compatibility)
- Rust 1.7x (Edition 2024) + `tokio` (async/sleep), `chrono` (timestamps), `clap` (CLI args) (018-fix-packet-granularity)
- Rust 1.7x (Edition 2024) + `rsudp-rust` internal modules (`trigger`, `pipeline`) (019-restore-sta-lta-alert)
- Rust 1.7x (Edition 2024) + `std::process::Command` (process spawning), `tempfile` (isolated environments), `regex` (log parsing), `tokio` (async test runner), `port_selector` or equivalent logic (dynamic ports). (020-add-e2e-alert-test)
- Temporary directories for test outputs. (020-add-e2e-alert-test)
- Rust 1.7x (Edition 2024) + None (021-cleanup-project-files)
- Filesystem (021-cleanup-project-files)
- local filesystem (temporary PNGs), Amazon S3 (public image hosting). (023-sns-posting)
- Rust 1.7x (Edition 2024) + None (Standard Library + internal logic) (024-sta-lta-fix)
- Log files in `logs/` directory. (026-rsudp-test-run)
- Rust 1.7x, Node.js 22, Docker, Docker Compose + `cargo`, `npm`, `docker`, `docker compose` (030-add-docker-support)
- Docker Volumes (for config, logs, and data persistence) (030-add-docker-support)
- Rust 1.7x (Edition 2021) + `tokio`, `reqwest`, `serde`, `clap`. + `mdns-sd` (for discovery), `reqwest` (for API v2/HTTPS), `clap` (for CLI). (031-hue-v2-integration)
- `rsudp.toml` (plain text `app_key` and config). (031-hue-v2-integration)
- Rust 1.7x (Edition 2021) + `tokio`, `rodio`, `serde`, `config`. + `rodio` (for audio), `tokio::task::spawn_blocking`. (032-alert-audio-playback)
- `rsudp.toml` (configuration for file paths). (032-alert-audio-playback)
- Rust 1.7x (Edition 2021) + `tokio` (async runtime, `UdpSocket`, `mpsc`), `tracing` (logging), `serde` (config deserialization) — すべて既存依存 (034-udp-forward)
- N/A (インメモリ統計のみ) (034-udp-forward)
- Rust 1.7x (latest stable) + Edition 2021 + `tokio` (async runtime, net, time), `tracing` (logging), `serde` (serialization) — all already in Cargo.toml (035-rsam-calculation)
- In-memory sliding window buffer (Vec<f64> with time-based eviction) (035-rsam-calculation)
- Rust 1.7x (Edition 2021) + TypeScript 5 / Next.js 14.2 (036-webui-spectrogram)
- In-memory ring buffers (server-side `VecDeque`, client-side `Float32Array`) (036-webui-spectrogram)
- TypeScript (Next.js 14+) for frontend; Rust backend unchanged + Next.js, React, Tailwind CSS (all existing) (037-webui-plot-polish)
- N/A (UI-only changes) (037-webui-plot-polish)

- Rust 1.7x (latest stable) + None (requires Rust toolchain: `rustc`, `cargo`) (001-init-rust-project)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust 1.7x (latest stable): Follow standard conventions

## Recent Changes
- 037-webui-plot-polish: Added TypeScript (Next.js 14+) for frontend; Rust backend unchanged + Next.js, React, Tailwind CSS (all existing)
- 036-webui-spectrogram: Added Rust 1.7x (Edition 2021) + TypeScript 5 / Next.js 14.2
- 036-webui-spectrogram: Added Rust 1.7x (Edition 2021) + TypeScript 5 / Next.js 14.2


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
