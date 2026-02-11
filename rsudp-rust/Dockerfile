# Stage 1: Builder
FROM rust:bookworm AS builder

RUN apt-get update && apt-get install -y clang libclang-dev libasound2-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Stage 2: Runtime
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

# Copy the binary
COPY --from=builder /usr/src/app/target/release/rsudp-rust /app/rsudp-rust

# Copy ALSA shared library required at runtime
COPY --from=builder /usr/lib/x86_64-linux-gnu/libasound.so.2* /usr/lib/x86_64-linux-gnu/

# Use the non-root user (uid 65532) defined in distroless
USER nonroot

# Expose the UDP port
EXPOSE 8888/udp

# Set the entrypoint
ENTRYPOINT ["/app/rsudp-rust", "--config=/etc/rsudp/rsudp.toml"]
