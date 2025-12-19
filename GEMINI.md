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
- 007-webui-plot: Added Rust 1.7x (Backend), TypeScript / Next.js 14+ (Frontend)
- 007-webui-plot: Added [if applicable, e.g., PostgreSQL, CoreData, files or N/A]
- 006-sta-lta-alert: Added Rust 1.7x + `tokio` (async runtime), `chrono` (time handling), `thiserror` (error handling), `byteorder` (parsing). Verification requires Python 3.x + `obspy`.


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
