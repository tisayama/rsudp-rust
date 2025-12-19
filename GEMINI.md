# rustrsudp_speckit Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-12-19

## Active Technologies
- Rust 1.7x (latest stable) + `tokio` (async runtime, net), `clap` (CLI args), `tracing` (logging) (002-udp-receiver)
- In-memory queue (Tokio MPSC channel) (002-udp-receiver)
- Rust 1.7x (latest stable) + None (standard library only for calculation). Python 3.x + `obspy` required for verification tests. (003-sta-lta-calc)
- N/A (Processing logic only) (003-sta-lta-calc)

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
- 003-sta-lta-calc: Added Rust 1.7x (latest stable) + None (standard library only for calculation). Python 3.x + `obspy` required for verification tests.
- 002-udp-receiver: Added Rust 1.7x (latest stable) + `tokio` (async runtime, net), `clap` (CLI args), `tracing` (logging)

- 001-init-rust-project: Added Rust 1.7x (latest stable) + None (requires Rust toolchain: `rustc`, `cargo`)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
