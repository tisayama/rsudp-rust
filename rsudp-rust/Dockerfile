# Stage 1: Builder
FROM rust:bookworm AS builder

RUN apt-get update && apt-get install -y clang libclang-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Stage 2: Runtime
FROM gcr.io/distroless/cc-debian12

# Use the non-root user (uid 65532) defined in distroless
USER nonroot

WORKDIR /app

# Copy the binary
COPY --from=builder /usr/src/app/target/release/rsudp-rust /app/rsudp-rust

# Expose the UDP port
EXPOSE 8888/udp

# Set the entrypoint
ENTRYPOINT ["/app/rsudp-rust"]
