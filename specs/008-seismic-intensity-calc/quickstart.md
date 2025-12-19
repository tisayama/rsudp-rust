# Quickstart: Japanese Seismic Intensity Calculation

## Integration

The `IntensityManager` is a standalone component that consumes synchronized samples from 3 components and calculates the JMA intensity.

### 1. Configuration

```rust
let config = IntensityConfig {
    channels: ["ENE".into(), "ENN".into(), "ENZ".into()],
    sample_rate: 100.0,
    sensitivities: [167000.0, 167000.0, 167000.0], // Counts per Gal
};
```

### 2. Processing Samples

As synchronized samples arrive from the data ingestion pipeline:

```rust
manager.add_samples(samples_map); // samples_map: HashMap<ChannelID, Vec<f64>>

if manager.should_calculate(current_time) {
    let result = manager.calculate(current_time)?;
    println!("Intensity: {} ({})", result.instrumental_intensity, result.intensity_class);
    
    // Broadcast to WebUI
    broadcast_tx.send(WsMessage::Intensity(result));
}
```

## Running Verification with MiniSEED

To verify against the reference data provided in this repository:

```bash
cd rsudp-rust
cargo run -- --file ../references/mseed/fdsnws.mseed --channels ENE,ENN,ENZ
```

Expected output should contain:
`計測震度: X.XX (Y)`
where Y is the JMA intensity class.

### Troubleshooting
- **Address in Use**: If you get a port error, use `--web-port 8081` to use a different port for the WebUI.
- **Missing Data**: Ensure you have at least 60 seconds of data for all 3 channels before calculation starts.