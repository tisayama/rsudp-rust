# Research: Data Ingestion Pipeline

## Decision: MiniSEED Parsing Strategy

- **Decision**: Use an existing MiniSEED library (e.g., `mseed` crate) or provide a basic robust parser if only specific records are needed. Given the "full compliance" requirement, using a library that wraps `libmseed` is the most reliable approach for handling Steim compression and various record types.
- **Rationale**: Re-implementing MiniSEED (especially Steim decompression) in Pure Rust is highly complex and error-prone. A library-based approach ensures stability and correctness, aligning with the project constitution.
- **Alternatives Considered**: 
    - Pure Rust implementation: Rejected due to development cost and risk of bugs in complex decompression logic.
    - Simple fixed-format parser: Rejected because the user specifically requested full-spec compliance if possible.

## Decision: NSLC State Management

- **Decision**: Use `DashMap` (concurrent hash map) or `std::collections::HashMap` wrapped in a `Mutex` to manage filter states.
- **Rationale**: Different data sources (identified by NSLC) need independent STA/LTA states. A hash map allows dynamic creation and lookup of these states as packets arrive.
- **Key Format**: `String` concatenating "Network.Station.Location.Channel".

## Decision: Pipeline Architecture

- **Decision**: Implement a 3-stage asynchronous pipeline using Tokio MPSC channels.
    1.  **Ingestor**: Reads from UDP or Files.
    2.  **Parser**: Decodes MiniSEED records into raw samples + metadata.
    3.  **Processor**: Updates STA/LTA filters based on metadata.
- **Rationale**: Separating these concerns allows for high throughput and easier testing of individual stages (e.g., testing the Parser with files).
