# Quickstart: Pure Rust MiniSEED Ingestion

This guide explains how to switch to and verify the Pure Rust MiniSEED parser.

## Verification Steps

1. **Remove C Dependencies**:
   Uninstall `clang` or `build-essential` temporarily to ensure no C-based components are used during the build.

2. **Run Build**:
   ```bash
   cargo build
   ```
   The build should succeed without any FFI-related compilation.

3. **Validate Accuracy**:
   Compare the output of the new parser against the reference data generated in `004`.
   ```bash
   cargo run -- --file references/mseed/fdsnws.mseed
   ```
   Check logs for parsed segments and verify the number of samples matches exactly.

4. **Integration Test**:
   The existing integration tests in `tests/` should pass with the new implementation.
   ```bash
   cargo test
   ```
