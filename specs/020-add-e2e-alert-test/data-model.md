# Data Model: E2E Test Structure

## Entities

### `TestContext`
Manages the lifecycle of the test environment.

| Field | Type | Description |
|-------|------|-------------|
| temp_dir | TempDir | Working directory for logs and outputs |
| udp_port | u16 | Dynamically assigned port |
| rsudp_process | Child | Handle to rsudp-rust process |
| streamer_process | Option<Child> | Handle to streamer process |

## Process Flow

1. **Setup**:
   - Create `TempDir`.
   - Find free UDP port.
   - Start `rsudp-rust` with `--udp-port PORT`, `--output-dir TEMP_DIR`.
   - Redirect stdout/stderr to `TEMP_DIR/rsudp.log`.

2. **Execution**:
   - Start `streamer` with `--addr 127.0.0.1:PORT`, `--speed 100.0`.
   - Wait for `streamer` to exit (success) or timeout.

3. **Verification**:
   - Poll `rsudp.log` for "ALARM".
   - Check `TEMP_DIR/alerts/` for PNG files.

4. **Teardown** (via Drop):
   - Kill `rsudp_process`.
   - Kill `streamer_process` (if running).
   - `TempDir` cleans itself up.
