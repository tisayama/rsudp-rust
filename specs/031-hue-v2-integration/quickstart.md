# Quickstart: Philips Hue Integration

## Prerequisites
- Philips Hue Bridge (v2/Square) connected to local network.
- `rsudp-hue` CLI tool (installed with `make install` or `cargo build`).

## Setup Workflow

1.  **Discover Bridge**:
    Run the setup tool to find your bridge IP and ID.
    ```bash
    rsudp-hue setup
    ```
    *Follow the on-screen instructions to press the Link Button on your bridge.*

2.  **Save Credentials**:
    The tool will output an `app_key`. Copy this key.

3.  **Find Lights**:
    List available lights to get their IDs.
    ```bash
    rsudp-hue list --ip <BRIDGE_IP> --key <APP_KEY>
    ```

4.  **Configure `rsudp.toml`**:
    Add the `[HUE]` section to your config file:
    ```toml
    [HUE]
    enabled = true
    app_key = "YOUR_GENERATED_APP_KEY"
    bridge_id = "YOUR_BRIDGE_ID" # Optional, for tracking IP changes
    target_ids = [
      "uuid-of-light-1",
      "uuid-of-light-2"
    ]
    ```

5.  **Restart Service**:
    ```bash
    sudo systemctl restart rsudp
    ```

## Testing

To test the integration without a real earthquake:

1.  **Trigger Simulation**:
    Send a UDP packet or use a test script to trigger a high STA/LTA ratio.
    
2.  **Verify**:
    - Lights should pulse **Yellow** immediately.
    - After the event simulation ends (Reset), lights should pulse **Colored** (based on intensity) for 20 seconds.
    - Lights should return to their original state.
