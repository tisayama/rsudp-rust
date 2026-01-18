# Research: Automated E2E Alert Triggering Test

## Decisions

### Decision: Dynamic Port Selection
**Rationale**: Hardcoding ports (8888/8889) causes conflicts in CI or parallel test runs.
**Solution**: Use `std::net::UdpSocket::bind("127.0.0.1:0")` to let the OS assign a free port, get the port number, and pass it to the subprocesses via CLI arguments (`--udp-port`, `--addr`). Since `rsudp-rust` binds 0.0.0.0, we just need to find a free port first. However, binding and closing creates a race condition.
**Refined Solution**: A safer approach is to try binding a random port in a loop or rely on a crate like `port_selector` if available, or just bind-and-release and hope for no race (common in simple tests). Given `rsudp-rust` takes a port argument, we will try to find a free port.

### Decision: Process Management
**Rationale**: `std::process::Command` does not automatically kill children when the parent drops.
**Solution**: Wrap `Child` in a struct that implements `Drop` to send `SIGTERM`/`kill` to the process. This ensures cleanup even on test panic.

### Decision: Log Assertion
**Rationale**: Need to verify "ALARM" log message.
**Solution**: Capture `stdout` and `stderr` of `rsudp-rust` using `Stdio::piped()`. Spawn a thread to read the pipe and check for the string/regex. Or simpler: redirect to a temp file and read it after the test. Given the async nature, reading a temp file after completion (or polling it) is robust and easy to debug.

## Research Tasks

### Task: Dependency Check
**Decision**: Check if `regex` or `port_selector` are already in `Cargo.toml`. If not, add them to `[dev-dependencies]`.
