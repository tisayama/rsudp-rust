# Quickstart: Running the E2E Alert Test

## Prerequisites
- Rust toolchain installed.

## Running the Test

Execute the newly added integration test:

```bash
cd rsudp-rust
cargo test --test e2e_alert -- --nocapture
```

## Expected Output

```text
running 1 test
test test_e2e_alert_triggering ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.23s
```

If the test fails, check the output for logs from `rsudp-rust`.
