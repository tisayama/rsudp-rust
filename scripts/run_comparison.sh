#!/bin/bash
set -e

# T008: Driver script

# 1. Setup
./scripts/setup_python_env.sh

# 2. Run Python Reference
./scripts/run_python_ref.sh

# 3. Run Rust Target
./scripts/run_rust_target.sh

# 4. Compare
echo "Generating comparison report..."
source rsudp-venv/bin/activate
python3 scripts/compare_logs.py logs/rsudp_python.log logs/rsudp_rust.log logs/comparison_report.csv

echo "Done. Check logs/comparison_report.csv"
cat logs/comparison_report.csv
