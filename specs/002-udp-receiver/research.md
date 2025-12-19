# Research: UDP Packet Reception

## Decision: Use `tokio` for Async I/O

- **Decision**: Utilize the `tokio` crate for the asynchronous runtime and UDP socket handling.
- **Rationale**: `tokio` is the de facto standard for async Rust. It provides high-performance, non-blocking network I/O, which is crucial for handling high-frequency sensor data without dropping packets. It also offers robust channels (`mpsc`) for the required in-memory queueing.
- **Alternatives Considered**: 
    - `std::net::UdpSocket` (Blocking): Rejected because blocking I/O would require managing threads manually to handle high concurrency, increasing complexity and resource usage.
    - `async-std`: A viable alternative, but `tokio` has a larger ecosystem and community support, which aligns better with long-term maintainability.

## Decision: Use `clap` for Configuration

- **Decision**: Use `clap` (derive feature) for parsing command-line arguments (port number).
- **Rationale**: `clap` provides a type-safe, declarative way to define CLI arguments. It automatically generates help messages and handles validation, improving the user experience and code clarity.
- **Alternatives Considered**: Manually parsing `std::env::args`. Rejected due to brittleness and lack of built-in validation/help generation.

## Decision: Use `tracing` for Observability

- **Decision**: Use `tracing` for structured logging.
- **Rationale**: `tracing` allows for asynchronous, structured logging that is essential for debugging concurrent systems like a UDP receiver.
- **Alternatives Considered**: `log` crate. Rejected because `tracing` offers superior context management (spans) for async workflows.
