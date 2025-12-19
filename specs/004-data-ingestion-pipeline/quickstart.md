# Quickstart: Data Ingestion Pipeline

This guide explains how to use the ingestion pipeline in simulation mode.

## Prerequisites

- One or more MiniSEED files (e.g., `sample.mseed`).

## Running in Simulation Mode

You can feed files into the pipeline using the `--file` (or `-f`) argument.

```bash
cargo run -- --file tests/data/earthquake.mseed
```

Multiple files:
```bash
cargo run -- --file file1.mseed --file file2.mseed
```

## Running in Real-time Mode

The default mode remains UDP ingestion.

```bash
cargo run -- --port 8888
```

## Expected Output

You should see logs indicating the number of samples parsed and the resulting STA/LTA ratios (if high enough to be interesting).

Example:
```text
INFO rsudp_rust::pipeline: Parsed segment AM.R1234.00.SHZ (100 samples)
INFO rsudp_rust::pipeline: Processing samples... current ratio: 1.2
```
