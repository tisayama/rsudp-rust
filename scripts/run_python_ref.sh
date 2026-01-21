#!/bin/bash
set -e

# Setup env
source rsudp-venv/bin/activate

# Copy config to default location where rsudp looks for it
mkdir -p ~/.config/rsudp
cp scripts/rsudp_settings.json ~/.config/rsudp/rsudp_settings.json

echo "Starting Python rsudp reference..."
# Run rsudp client in background, redirecting stdout/stderr
# Note: client.py might need specific arguments or just rely on settings.json in its dir
# Assuming running from repo root, verify import paths
export PYTHONPATH=$PYTHONPATH:$(pwd)/references/rsudp

# Start rsudp client (headless if possible, or expect GUI)
# Since we are in a headless environment, this might fail if it tries to open a window.
# We hope it logs to stdout before crashing or can run headless.
# If rsudp requires GUI, we might need Xvfb or skip. Assuming it works or logs enough.
python3 references/rsudp/rsudp/client.py > logs/rsudp_python.log 2>&1 &
RSUDP_PID=$!

echo "Waiting for rsudp to initialize..."
sleep 10

echo "Streaming data to Python rsudp..."
# Send data to localhost:8888 (default rsudp port)
./rsudp-rust/target/release/streamer --file references/mseed/fdsnws.mseed --addr 127.0.0.1:8888 --speed 100.0

echo "Stream complete. Waiting for processing..."
sleep 5

kill $RSUDP_PID || true
echo "Python run complete."
