# Quickstart: UDP Receiver

This guide explains how to run and test the UDP receiver feature.

## Prerequisites

- `netcat` (nc) tool installed (for sending test packets).

## Running the Application

1.  **Build the project**:
    ```bash
    cargo build
    ```

2.  **Run the receiver**:
    By default, it listens on port 8888.
    ```bash
    cargo run
    ```
    
    To specify a custom port:
    ```bash
    cargo run -- --port 9999
    ```

## Testing Reception

1.  **Open a second terminal**.

2.  **Send a UDP packet**:
    ```bash
    echo "Hello Seismic World" | nc -u -w 1 127.0.0.1 8888
    ```

3.  **Verify Output**:
    Check the terminal running `rsudp-rust`. You should see a log message indicating a packet was received, including its size and source.

    Example log:
    ```text
    INFO rsudp_rust::receiver: Received 20 bytes from 127.0.0.1:xxxxx
    ```
