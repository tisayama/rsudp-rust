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

# Start Python rsudp in background
# Use 10101 port
rsudp-venv/bin/python references/rsudp/rsudp/client.py -s scripts/rsudp_settings.json > logs/rsudp_python.log 2>&1 &
RSUDP_PID=$!

sleep 5

echo "Streaming data to Python rsudp..."
# Send data to localhost:10101
./rsudp-rust/target/release/streamer --file references/mseed/fdsnws.mseed --addr 127.0.0.1:10101 --speed 1.0

sleep 250; kill -9 $RSUDP_PID; echo "Finished 4m run"
sleep 5

kill $RSUDP_PID || true
echo "Python run complete."
