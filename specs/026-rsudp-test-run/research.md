# Research: RSUDP Test Run and Comparison

## Decisions

### Decision: Python Virtual Environment (`venv`)
**Rationale**: `rsudp` requires specific versions of `obspy`, `matplotlib`, and `numpy`. Installing these globally can break other tools. A project-local `venv` is isolated and safe.
**Choice**: Create `rsudp-venv` in the project root, install `requirements.txt` from `references/rsudp`, and run the reference implementation within this environment.

### Decision: Log Parsing Strategy
**Rationale**: We need to extract timestamps and messages from unstructured text logs.
**Choice**: Use a simple Python script (`compare_logs.py`) with regex to find lines containing "Trigger threshold ... exceeded" and "Earthquake trigger reset". The script will parse timestamps and output a CSV.

### Decision: Test Data
**Rationale**: `fdsnws.mseed` is short. To test stability over 10 minutes, we need to loop it or verify that short-term behavior matches exactly. Given the user's request for "10 minute run" validation, we will loop the `streamer` for Rust, but for Python `rsudp` we might need to rely on its internal file reading or set up a UDP feed to it.
**Refinement**: `rsudp` can read from UDP. We can use our Rust `streamer` to send data to *both* the Rust receiver and the Python receiver (sequentially or in parallel).
**Choice**: Run `streamer` targeting Python `rsudp` first, then run `streamer` targeting `rsudp-rust`. This ensures identical input timing and content.

## Research Tasks

- [x] Identify `rsudp` requirements (Confirmed: `obspy`, `matplotlib`, `numpy` etc. in `references/rsudp/docsrc/requirements.txt`).
- [ ] Create `compare_logs.py` script (Tasks phase).
- [ ] Create `run_comparison.sh` driver script (Tasks phase).
