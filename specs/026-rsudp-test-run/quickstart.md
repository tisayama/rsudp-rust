# Quickstart: Running the Comparison

## Prerequisites
- Rust toolchain
- Python 3.x
- `venv` module

## Automated Run

1. **Execute the comparison script**:
   ```bash
   ./scripts/run_comparison.sh
   ```

2. **View Results**:
   ```bash
   cat logs/comparison_report.csv
   ```

## Manual Steps

1. **Setup Python Env**:
   ```bash
   python3 -m venv rsudp-venv
   source rsudp-venv/bin/activate
   pip install -r references/rsudp/docsrc/requirements.txt
   ```

2. **Run Python rsudp**:
   ```bash
   # Configure settings.json first
   python references/rsudp/rsudp/client.py > logs/rsudp_python.log 2>&1 &
   # Start streamer
   ./target/release/streamer ...
   ```

3. **Run Rust rsudp**:
   ```bash
   # Configure settings.toml first
   ./target/release/rsudp-rust > logs/rsudp_rust.log 2>&1 &
   # Start streamer
   ./target/release/streamer ...
   ```
