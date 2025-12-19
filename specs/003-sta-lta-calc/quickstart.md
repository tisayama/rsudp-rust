# Quickstart: STA/LTA Calculation

This guide explains how to verify the STA/LTA calculation logic.

## Prerequisites

- Python 3.x
- `obspy` and `numpy` installed: `pip install obspy numpy`

## Running Tests

1.  **Generate Reference Data**:
    A Python script is provided to generate reference data for testing.
    (This is handled automatically by the test suite if configured, or can be run manually).

    ```bash
    # Example manual generation (if implemented as such)
    python3 rsudp-rust/tests/scripts/generate_stalta_reference.py > reference.csv
    ```

2.  **Run Rust Tests**:
    ```bash
    cargo test
    ```
    This will execute the unit tests, which calculate STA/LTA values and compare them against the expected results.

## Usage Example (Rust)

```rust
use rsudp_rust::trigger::RecursiveStaLta;

let nsta = 100; // 1 second at 100Hz
let nlta = 1000; // 10 seconds at 100Hz
let mut stalta = RecursiveStaLta::new(nsta, nlta);

let data = vec![0.1, 0.2, 0.5, ...];
for sample in data {
    let ratio = stalta.process(sample);
    println!("Ratio: {}", ratio);
}
```
