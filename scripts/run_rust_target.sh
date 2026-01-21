#!/bin/bash
set -e

echo "Starting Rust rsudp target..."
# Use the toml settings created earlier
# Note: rsudp-rust takes args to override, or loads from default location.
# We'll use CLI args to enforce the test config if the config file loader isn't flexible,
# or assume we can pass -c. The main.rs supports -C/--config.

./rsudp-rust/target/release/rsudp-rust --config scripts/rsudp_settings.toml > logs/rsudp_rust.log 2>&1 &
RUST_PID=$!

echo "Waiting for rsudp-rust to initialize..."
sleep 5

echo "Streaming data to Rust rsudp..."
./rsudp-rust/target/release/streamer --file references/mseed/fdsnws.mseed --addr 127.0.0.1:9999 --speed 100.0

echo "Stream complete. Waiting for processing..."
sleep 5

kill $RUST_PID || true
echo "Rust run complete."
