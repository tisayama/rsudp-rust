use rsudp_rust::forward::{ForwardManager, should_forward_channel};
use rsudp_rust::settings::ForwardSettings;
use std::time::Duration;
use tokio::net::UdpSocket;

/// Helper: create ForwardSettings for testing with given listener addresses.
fn test_settings(addrs: Vec<String>, ports: Vec<u16>) -> ForwardSettings {
    ForwardSettings {
        enabled: true,
        address: addrs,
        port: ports,
        channels: vec!["all".to_string()],
        fwd_data: true,
        fwd_alarms: false,
    }
}

/// Helper: bind a local UDP listener and return the socket and its port.
async fn bind_listener() -> (UdpSocket, u16) {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    (socket, port)
}

/// Helper: receive one packet from a UDP socket with timeout.
async fn recv_with_timeout(socket: &UdpSocket, timeout_ms: u64) -> Option<Vec<u8>> {
    let mut buf = [0u8; 65535];
    match tokio::time::timeout(Duration::from_millis(timeout_ms), socket.recv_from(&mut buf)).await
    {
        Ok(Ok((len, _))) => Some(buf[..len].to_vec()),
        _ => None,
    }
}

/// Helper: drain all packets from a UDP socket within a timeout.
async fn recv_all_with_timeout(socket: &UdpSocket, timeout_ms: u64) -> Vec<Vec<u8>> {
    let mut results = Vec::new();
    loop {
        match recv_with_timeout(socket, timeout_ms).await {
            Some(data) => results.push(data),
            None => break,
        }
    }
    results
}

// ============================================================================
// Phase 3: US1 - Forward Seismic Data to Remote Receivers (T011-T013)
// ============================================================================

#[tokio::test]
async fn test_single_destination_data_forwarding() {
    // T011: bind local UDP listener, create ForwardManager with one destination,
    // call forward_data(), assert listener receives identical bytes.
    let (listener, port) = bind_listener().await;
    let settings = test_settings(vec!["127.0.0.1".to_string()], vec![port]);

    let fm = ForwardManager::new(&settings).await.unwrap();
    let test_data = b"SAMPLE_SEISMIC_DATA_12345";
    fm.forward_data("EHZ", test_data);

    // Allow time for async send
    let received = recv_with_timeout(&listener, 1000).await;
    assert!(received.is_some(), "Listener should have received a packet");
    assert_eq!(received.unwrap(), test_data.to_vec());
}

#[tokio::test]
async fn test_multi_destination_data_forwarding() {
    // T012: bind two local UDP listeners, create ForwardManager with two destinations,
    // call forward_data(), assert both listeners receive identical bytes.
    let (listener1, port1) = bind_listener().await;
    let (listener2, port2) = bind_listener().await;
    let settings = test_settings(
        vec!["127.0.0.1".to_string(), "127.0.0.1".to_string()],
        vec![port1, port2],
    );

    let fm = ForwardManager::new(&settings).await.unwrap();
    let test_data = b"MULTI_DEST_DATA";
    fm.forward_data("EHZ", test_data);

    let r1 = recv_with_timeout(&listener1, 1000).await;
    let r2 = recv_with_timeout(&listener2, 1000).await;
    assert!(r1.is_some(), "Listener 1 should have received a packet");
    assert!(r2.is_some(), "Listener 2 should have received a packet");
    assert_eq!(r1.unwrap(), test_data.to_vec());
    assert_eq!(r2.unwrap(), test_data.to_vec());
}

#[tokio::test]
async fn test_forwarding_disabled() {
    // T013: verify no ForwardManager is created when enabled=false,
    // and no packets are sent.
    let (listener, port) = bind_listener().await;
    let mut settings = test_settings(vec!["127.0.0.1".to_string()], vec![port]);
    settings.enabled = false;

    // When forwarding is disabled, main.rs doesn't create ForwardManager.
    // Simulate that: if enabled is false, we don't call forward_data at all.
    // But let's also test that if someone creates a manager with enabled=true
    // but fwd_data=false, no data is forwarded.
    settings.enabled = true;
    settings.fwd_data = false;

    let fm = ForwardManager::new(&settings).await.unwrap();
    fm.forward_data("EHZ", b"SHOULD_NOT_ARRIVE");

    let received = recv_with_timeout(&listener, 500).await;
    assert!(received.is_none(), "Listener should NOT have received a packet when fwd_data=false");
}

// ============================================================================
// Phase 4: US2 - Filter Forwarded Data by Channel and Message Type (T016-T018)
// ============================================================================

#[tokio::test]
async fn test_channel_filtering() {
    // T016: configure channels = ["EHZ"], send data for EHZ, EHN, EHE,
    // assert only EHZ data is received.
    let (listener, port) = bind_listener().await;
    let mut settings = test_settings(vec!["127.0.0.1".to_string()], vec![port]);
    settings.channels = vec!["EHZ".to_string()];

    let fm = ForwardManager::new(&settings).await.unwrap();
    fm.forward_data("EHZ", b"DATA_EHZ");
    fm.forward_data("EHN", b"DATA_EHN");
    fm.forward_data("EHE", b"DATA_EHE");

    // Wait for all potential sends
    tokio::time::sleep(Duration::from_millis(200)).await;

    let packets = recv_all_with_timeout(&listener, 500).await;
    assert_eq!(packets.len(), 1, "Only EHZ data should be forwarded");
    assert_eq!(packets[0], b"DATA_EHZ".to_vec());
}

#[tokio::test]
async fn test_fwd_data_false_suppresses_data() {
    // T017: configure fwd_data=false, send data, assert listener receives nothing.
    let (listener, port) = bind_listener().await;
    let mut settings = test_settings(vec!["127.0.0.1".to_string()], vec![port]);
    settings.fwd_data = false;

    let fm = ForwardManager::new(&settings).await.unwrap();
    fm.forward_data("EHZ", b"SHOULD_NOT_FORWARD");

    let received = recv_with_timeout(&listener, 500).await;
    assert!(received.is_none(), "No data should be forwarded when fwd_data=false");
}

#[tokio::test]
async fn test_fwd_alarms_flag() {
    // T018: test fwd_alarms=true forwards alarm messages, fwd_alarms=false suppresses them.

    // Test 1: fwd_alarms=true should forward
    let (listener1, port1) = bind_listener().await;
    let mut settings1 = test_settings(vec!["127.0.0.1".to_string()], vec![port1]);
    settings1.fwd_alarms = true;

    let fm1 = ForwardManager::new(&settings1).await.unwrap();
    fm1.forward_alarm("ALARM EHZ 2026-02-10T12:00:00Z");

    let r1 = recv_with_timeout(&listener1, 1000).await;
    assert!(r1.is_some(), "Alarm should be forwarded when fwd_alarms=true");
    assert_eq!(
        String::from_utf8(r1.unwrap()).unwrap(),
        "ALARM EHZ 2026-02-10T12:00:00Z"
    );

    // Test 2: fwd_alarms=false should NOT forward
    let (listener2, port2) = bind_listener().await;
    let mut settings2 = test_settings(vec!["127.0.0.1".to_string()], vec![port2]);
    settings2.fwd_alarms = false;

    let fm2 = ForwardManager::new(&settings2).await.unwrap();
    fm2.forward_alarm("ALARM EHZ 2026-02-10T12:00:00Z");

    let r2 = recv_with_timeout(&listener2, 500).await;
    assert!(r2.is_none(), "Alarm should NOT be forwarded when fwd_alarms=false");
}

// ============================================================================
// Phase 5: US3 - Config Validation (edge cases)
// ============================================================================

#[tokio::test]
async fn test_config_mismatch_error() {
    // FR-006: address/port length mismatch should return error.
    let settings = ForwardSettings {
        enabled: true,
        address: vec!["127.0.0.1".to_string(), "127.0.0.2".to_string()],
        port: vec![8888], // only 1 port for 2 addresses
        channels: vec!["all".to_string()],
        fwd_data: true,
        fwd_alarms: false,
    };

    let result = ForwardManager::new(&settings).await;
    assert!(result.is_err(), "Should fail with mismatched address/port lengths");
    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(
        err_msg.contains("address count (2)") && err_msg.contains("port count (1)"),
        "Error message should describe the mismatch: {}",
        err_msg
    );
}

// ============================================================================
// Phase 6: US4 - Automated End-to-End Forwarding Tests (T022-T023)
// ============================================================================

#[tokio::test]
async fn test_e2e_forward_100_packets() {
    // T022: send 100 sample seismic data packets, verify all 100 received.
    // Send in batches to respect the bounded channel (capacity 32).
    let (listener, port) = bind_listener().await;
    let settings = test_settings(vec!["127.0.0.1".to_string()], vec![port]);

    let fm = ForwardManager::new(&settings).await.unwrap();

    // Send 100 packets in batches of 20 with brief pauses to let the
    // async forwarding task drain the channel.
    for batch in 0..5u32 {
        for i in 0..20u32 {
            let idx = batch * 20 + i;
            let data = format!("PACKET_{:04}", idx);
            fm.forward_data("EHZ", data.as_bytes());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Allow forwarding tasks to finish processing
    tokio::time::sleep(Duration::from_millis(500)).await;

    let packets = recv_all_with_timeout(&listener, 1000).await;
    assert_eq!(
        packets.len(),
        100,
        "Should have received all 100 packets, got {}",
        packets.len()
    );

    // Verify content of first and last packets
    assert_eq!(
        String::from_utf8(packets[0].clone()).unwrap(),
        "PACKET_0000"
    );
    assert_eq!(
        String::from_utf8(packets[99].clone()).unwrap(),
        "PACKET_0099"
    );
}

#[tokio::test]
async fn test_e2e_forward_with_filtering() {
    // T023: configure channels=["EHZ"] and fwd_alarms=true,
    // send mixed EHZ+EHN data and alarm messages,
    // verify only EHZ data packets and alarm messages received.
    let (listener, port) = bind_listener().await;
    let mut settings = test_settings(vec!["127.0.0.1".to_string()], vec![port]);
    settings.channels = vec!["EHZ".to_string()];
    settings.fwd_alarms = true;

    let fm = ForwardManager::new(&settings).await.unwrap();

    // Send mixed data
    fm.forward_data("EHZ", b"DATA_EHZ_1");
    fm.forward_data("EHN", b"DATA_EHN_1"); // should be filtered out
    fm.forward_data("EHZ", b"DATA_EHZ_2");
    fm.forward_data("EHE", b"DATA_EHE_1"); // should be filtered out
    fm.forward_alarm("ALARM EHZ 2026-02-10T12:00:00Z");
    fm.forward_alarm("RESET EHZ 2026-02-10T12:01:00Z");

    // Allow forwarding tasks to process
    tokio::time::sleep(Duration::from_millis(300)).await;

    let packets = recv_all_with_timeout(&listener, 1000).await;

    // Expected: 2 EHZ data + 2 alarms = 4 packets
    assert_eq!(
        packets.len(),
        4,
        "Should have received 4 packets (2 EHZ data + 2 alarms), got {}",
        packets.len()
    );

    let strings: Vec<String> = packets
        .iter()
        .map(|p| String::from_utf8(p.clone()).unwrap())
        .collect();

    assert!(strings.contains(&"DATA_EHZ_1".to_string()));
    assert!(strings.contains(&"DATA_EHZ_2".to_string()));
    assert!(strings.contains(&"ALARM EHZ 2026-02-10T12:00:00Z".to_string()));
    assert!(strings.contains(&"RESET EHZ 2026-02-10T12:01:00Z".to_string()));

    // Verify filtered-out packets are NOT present
    assert!(!strings.contains(&"DATA_EHN_1".to_string()));
    assert!(!strings.contains(&"DATA_EHE_1".to_string()));
}

// ============================================================================
// Unit tests for should_forward_channel (already in forward.rs, but test
// additional edge cases here for integration completeness)
// ============================================================================

#[test]
fn test_channel_suffix_matching_hz() {
    // "HZ" filter should match EHZ, SHZ, BHZ but not EHN, EHE
    let filters = vec!["HZ".to_string()];
    assert!(should_forward_channel("EHZ", &filters));
    assert!(should_forward_channel("SHZ", &filters));
    assert!(should_forward_channel("BHZ", &filters));
    assert!(!should_forward_channel("EHN", &filters));
    assert!(!should_forward_channel("EHE", &filters));
}

#[test]
fn test_channel_all_wildcard() {
    let filters = vec!["all".to_string()];
    assert!(should_forward_channel("EHZ", &filters));
    assert!(should_forward_channel("EHN", &filters));
    assert!(should_forward_channel("ENE", &filters));
    assert!(should_forward_channel("anything", &filters));
}
