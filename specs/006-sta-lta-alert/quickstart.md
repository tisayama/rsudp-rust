# Quickstart: STA/LTA Alert System

## Integration

The `AlertManager` is designed to be integrated into the data pipeline. It consumes samples and emits events via a channel.

### 1. Define Configuration

```rust
let config = AlertConfig {
    sta_seconds: 5.0,
    lta_seconds: 30.0,
    threshold: 1.6,
    reset_threshold: 1.5,
    channel_id: "SHZ".to_string(),
    filter_config: Some(FilterConfig::bandpass(1.0, 20.0)),
    ..Default::default()
};
```

### 2. Initialize and Run

```rust
let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(100);
let mut manager = AlertManager::new(config, event_tx);

// In your data processing loop:
for sample in data_stream {
    manager.process_sample(sample).await?;
}
```

### 3. Handle Events

```rust
while let Some(event) = event_rx.recv().await {
    match event.event_type {
        AlertEventType::Alarm => println!("ALARM! Triggered at {}", event.timestamp),
        AlertEventType::Reset => println!("RESET. Max ratio: {}", event.max_ratio.unwrap()),
    }
}
```

## Running Verification Tests

To verify parity with ObsPy:

```bash
# Generate reference data using ObsPy
python3 tests/scripts/generate_stalta_reference.py target/reference.csv

# Run Rust tests
cargo test test_compare_with_obspy_exact
```
