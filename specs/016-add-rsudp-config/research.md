# Research: Configuration Management for rsudp-rust

## Decisions

### Decision: Use `config-rs` library
**Rationale**: `config-rs` is the standard Rust library for layered configuration. It natively supports TOML, YAML, JSON, Environment Variables, and custom sources. It handles the merging logic and priority layers (e.g., CLI > Env > File > Default) cleanly.
**Alternatives considered**:
- **Manual merging with Serde**: Too much boilerplate for 50+ fields and multiple layers.
- **`figment`**: Excellent but slightly more complex than needed for this project.

### Decision: TOML as preferred format
**Rationale**: Aligns with the Rust ecosystem (Cargo.toml) and user request for TOML/YAML. TOML is less prone to "indentation hell" than YAML for deeply nested structures like rsudp's.
**Alternatives considered**:
- **JSON**: rsudp's original format. While supported, it's less user-friendly for manual editing.

### Decision: Environment Variable naming convention
**Rationale**: Use `RUSTRSUDP_` prefix and double underscore `__` for nesting (e.g., `RUSTRSUDP_PLOT__ENABLED=true` maps to `plot.enabled`). This is the standard pattern for `config-rs`.

## Research Tasks

### Task: Verify `rsudp` v1.0+ default values
**Decision**: Extract exact defaults from `rsudp/rsudp/c_settings.py` and implement them in `Settings::default()`.
**Finding**: Already extracted significant defaults from the reference file during the specification phase.

### Task: Handle "all" in channel lists
**Decision**: In rsudp, `channels = ["all"]` is common. In Rust, we will use a custom enum or handle this specific string during processing.
**Rationale**: To maintain 1:1 compatibility.

### Task: Migration from JSON
**Decision**: Although not explicitly requested, since rsudp uses JSON, we should ensure our system can also read `.json` if needed, but the focus remains on TOML/YAML.
