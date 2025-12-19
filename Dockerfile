# --- Frontend Build Stage ---
FROM node:22-alpine AS frontend-builder
WORKDIR /app/webui
COPY webui/package*.json ./
RUN npm install
COPY webui/ ./
RUN npm run build

# --- Backend Build Stage ---
FROM rust:1.82-alpine AS backend-builder
RUN apk add --no-cache musl-dev
WORKDIR /app/rsudp-rust
COPY rsudp-rust/Cargo*.toml ./
# Dummy build to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY rsudp-rust/src/ ./src/
RUN cargo build --release

# --- Final Runner Stage ---
FROM alpine:latest
WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache libgcc nodejs npm

# Copy backend
COPY --from=backend-builder /app/rsudp-rust/target/release/rsudp-rust /app/rsudp-rust

# Copy frontend
COPY --from=frontend-builder /app/webui/.next /app/webui/.next
COPY --from=frontend-builder /app/webui/public /app/webui/public
COPY --from=frontend-builder /app/webui/node_modules /app/webui/node_modules
COPY --from=frontend-builder /app/webui/package.json /app/webui/package.json

# Entry script to run both
RUN echo '#!/bin/sh
/app/rsudp-rust &
cd /app/webui && npm run start' > /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

EXPOSE 8080 3000
CMD ["/app/entrypoint.sh"]
