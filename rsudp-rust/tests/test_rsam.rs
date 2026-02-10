use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::Duration;

use chrono::Utc;
use rsudp_rust::parser::TraceSegment;
use rsudp_rust::rsam::{RsamManager, RsamResult};
use rsudp_rust::settings::RsamSettings;

fn make_settings(port: u16) -> RsamSettings {
    RsamSettings {
        enabled: true,
        quiet: true,
        fwaddr: "127.0.0.1".to_string(),
        fwport: port,
        fwformat: "LITE".to_string(),
        channel: "HZ".to_string(),
        interval: 2,
        deconvolve: false,
        units: "VEL".to_string(),
    }
}

fn make_segment(channel: &str, station: &str, samples: Vec<f64>) -> TraceSegment {
    TraceSegment {
        network: "AM".to_string(),
        station: station.to_string(),
        location: "00".to_string(),
        channel: channel.to_string(),
        starttime: Utc::now(),
        samples,
        sampling_rate: 100.0,
    }
}

fn bind_listener() -> (UdpSocket, u16) {
    let listener = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    listener
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    (listener, port)
}

// ============================================================
// T011: Unit test: RSAM calculation correctness
// ============================================================
#[test]
fn test_rsam_calculation_correctness() {
    let (_, port) = bind_listener();
    let settings = make_settings(port);
    let mut mgr = RsamManager::new(&settings, HashMap::new()).unwrap();

    // Feed known sample values: absolute values will be [10, 20, 30, 40, 50]
    let seg = make_segment("EHZ", "TEST", vec![10.0, -20.0, 30.0, -40.0, 50.0]);
    mgr.process_segment(&seg);

    let result = mgr.calculate().unwrap();
    assert_eq!(result.station, "TEST");
    assert_eq!(result.channel, "EHZ");
    assert!((result.mean - 30.0).abs() < 0.01, "mean should be 30.0, got {}", result.mean);
    assert!((result.median - 30.0).abs() < 0.01, "median should be 30.0, got {}", result.median);
    assert!((result.min - 10.0).abs() < 0.01, "min should be 10.0, got {}", result.min);
    assert!((result.max - 50.0).abs() < 0.01, "max should be 50.0, got {}", result.max);
}

// ============================================================
// T012: Unit test: LITE format output
// ============================================================
#[test]
fn test_rsam_lite_format() {
    let result = RsamResult {
        station: "TEST".to_string(),
        channel: "EHZ".to_string(),
        mean: 30.0,
        median: 30.0,
        min: 10.0,
        max: 50.0,
    };
    let formatted = result.format_lite();
    assert_eq!(formatted, "stn:TEST|ch:EHZ|mean:30|med:30|min:10|max:50");
}

// ============================================================
// T013: Unit test: JSON format output
// ============================================================
#[test]
fn test_rsam_json_format() {
    let result = RsamResult {
        station: "TEST".to_string(),
        channel: "EHZ".to_string(),
        mean: 30.0,
        median: 30.0,
        min: 10.0,
        max: 50.0,
    };
    let formatted = result.format_json();
    // Parse as JSON to validate structure
    let v: serde_json::Value = serde_json::from_str(&formatted).unwrap();
    assert_eq!(v["station"], "TEST");
    assert_eq!(v["channel"], "EHZ");
    assert_eq!(v["mean"], 30.0);
    assert_eq!(v["median"], 30.0);
    assert_eq!(v["min"], 10.0);
    assert_eq!(v["max"], 50.0);
}

// ============================================================
// T014: Unit test: CSV format output
// ============================================================
#[test]
fn test_rsam_csv_format() {
    let result = RsamResult {
        station: "TEST".to_string(),
        channel: "EHZ".to_string(),
        mean: 30.0,
        median: 30.0,
        min: 10.0,
        max: 50.0,
    };
    let formatted = result.format_csv();
    assert_eq!(formatted, "TEST,EHZ,30,30,10,50");
}

// ============================================================
// T012b: format() dispatcher with unknown fallback
// ============================================================
#[test]
fn test_rsam_format_dispatcher() {
    let result = RsamResult {
        station: "TEST".to_string(),
        channel: "EHZ".to_string(),
        mean: 1.0,
        median: 2.0,
        min: 0.5,
        max: 3.0,
    };
    // Known formats
    assert!(result.format("LITE").starts_with("stn:"));
    assert!(result.format("JSON").starts_with('{'));
    assert!(result.format("CSV").contains(','));
    // Unknown falls back to LITE
    assert!(result.format("UNKNOWN").starts_with("stn:"));
}

// ============================================================
// T015: Integration test: UDP delivery
// ============================================================
#[test]
fn test_rsam_udp_delivery() {
    let (listener, port) = bind_listener();

    let mut settings = make_settings(port);
    settings.interval = 0; // Trigger immediately on first process_segment
    let mut mgr = RsamManager::new(&settings, HashMap::new()).unwrap();

    // Feed enough data to trigger calculation
    let seg = make_segment("EHZ", "R6E01", vec![100.0, -200.0, 300.0]);
    // Sleep briefly to ensure interval has passed (interval=0 means immediate)
    std::thread::sleep(Duration::from_millis(10));
    mgr.process_segment(&seg);

    // Read from listener
    let mut buf = [0u8; 4096];
    let (n, _) = listener.recv_from(&mut buf).unwrap();
    let received = std::str::from_utf8(&buf[..n]).unwrap();

    assert!(received.starts_with("stn:R6E01|ch:EHZ|"), "Got: {}", received);
}

// ============================================================
// T017: Unit test: channel filtering
// ============================================================
#[test]
fn test_rsam_channel_filtering() {
    let (_, port) = bind_listener();
    let settings = make_settings(port);
    let mut mgr = RsamManager::new(&settings, HashMap::new()).unwrap();

    // Feed EHZ (should match "HZ")
    let seg_ehz = make_segment("EHZ", "TEST", vec![10.0, 20.0, 30.0]);
    mgr.process_segment(&seg_ehz);

    // Feed EHN (should NOT match "HZ")
    let seg_ehn = make_segment("EHN", "TEST", vec![100.0, 200.0, 300.0]);
    mgr.process_segment(&seg_ehn);

    // Feed EHE (should NOT match "HZ")
    let seg_ehe = make_segment("EHE", "TEST", vec![1000.0, 2000.0, 3000.0]);
    mgr.process_segment(&seg_ehe);

    // Calculate should only contain EHZ data
    let result = mgr.calculate().unwrap();
    assert_eq!(result.channel, "EHZ");
    // mean of [10, 20, 30] = 20.0
    assert!((result.mean - 20.0).abs() < 0.01, "mean={}, expected 20.0 (only EHZ data)", result.mean);
}

// ============================================================
// T018: Unit test: suffix matching variations
// ============================================================
#[test]
fn test_rsam_suffix_matching_variations() {
    let (_, port) = bind_listener();

    // channel="HZ" should match EHZ and SHZ
    let settings = make_settings(port);
    let mut mgr = RsamManager::new(&settings, HashMap::new()).unwrap();

    let seg1 = make_segment("EHZ", "TEST", vec![10.0]);
    mgr.process_segment(&seg1);
    assert!(mgr.calculate().is_some(), "HZ should match EHZ");

    // New manager for SHZ test
    let (_, port2) = bind_listener();
    let settings2 = make_settings(port2);
    let mut mgr2 = RsamManager::new(&settings2, HashMap::new()).unwrap();

    let seg2 = make_segment("SHZ", "TEST", vec![10.0]);
    mgr2.process_segment(&seg2);
    assert!(mgr2.calculate().is_some(), "HZ should match SHZ");

    // channel="ENZ" should match ENZ but not ENE
    let (_, port3) = bind_listener();
    let mut settings3 = make_settings(port3);
    settings3.channel = "ENZ".to_string();
    let mut mgr3 = RsamManager::new(&settings3, HashMap::new()).unwrap();

    let seg3 = make_segment("ENZ", "TEST", vec![10.0]);
    mgr3.process_segment(&seg3);
    assert!(mgr3.calculate().is_some(), "ENZ should match ENZ");

    let (_, port4) = bind_listener();
    let mut settings4 = make_settings(port4);
    settings4.channel = "ENZ".to_string();
    let mut mgr4 = RsamManager::new(&settings4, HashMap::new()).unwrap();

    let seg4 = make_segment("ENE", "TEST", vec![10.0]);
    mgr4.process_segment(&seg4);
    assert!(mgr4.calculate().is_none(), "ENZ should not match ENE");
}

// ============================================================
// T022: Unit test: deconvolution with known sensitivity
// ============================================================
#[test]
fn test_rsam_deconvolution() {
    let (_, port) = bind_listener();
    let mut settings = make_settings(port);
    settings.deconvolve = true;
    settings.units = "VEL".to_string();

    let mut sens_map = HashMap::new();
    sens_map.insert("EHZ".to_string(), 1000.0);

    let mut mgr = RsamManager::new(&settings, sens_map).unwrap();

    // Feed [1000.0, -2000.0] → divided by 1000 → [1.0, 2.0] absolute
    let seg = make_segment("EHZ", "TEST", vec![1000.0, -2000.0]);
    mgr.process_segment(&seg);

    let result = mgr.calculate().unwrap();
    assert!((result.mean - 1.5).abs() < 0.01, "mean should be 1.5, got {}", result.mean);
    assert!((result.min - 1.0).abs() < 0.01, "min should be 1.0, got {}", result.min);
    assert!((result.max - 2.0).abs() < 0.01, "max should be 2.0, got {}", result.max);
}

// ============================================================
// T023: Unit test: deconvolution GRAV mode
// ============================================================
#[test]
fn test_rsam_deconvolution_grav() {
    let (_, port) = bind_listener();
    let mut settings = make_settings(port);
    settings.deconvolve = true;
    settings.units = "GRAV".to_string();

    let mut sens_map = HashMap::new();
    sens_map.insert("EHZ".to_string(), 100.0);

    let mut mgr = RsamManager::new(&settings, sens_map).unwrap();

    // Feed [981.0] → 981/100/9.81 ≈ 1.0
    let seg = make_segment("EHZ", "TEST", vec![981.0]);
    mgr.process_segment(&seg);

    let result = mgr.calculate().unwrap();
    assert!((result.mean - 1.0).abs() < 0.01, "mean should be ~1.0, got {}", result.mean);
}

// ============================================================
// T024: Unit test: deconvolution fallback (no sensitivity)
// ============================================================
#[test]
fn test_rsam_deconvolution_fallback() {
    let (_, port) = bind_listener();
    let mut settings = make_settings(port);
    settings.deconvolve = true;
    settings.units = "VEL".to_string();

    // Empty sensitivity map → should fall back to raw counts
    let mut mgr = RsamManager::new(&settings, HashMap::new()).unwrap();

    let seg = make_segment("EHZ", "TEST", vec![100.0, -200.0]);
    mgr.process_segment(&seg);

    let result = mgr.calculate().unwrap();
    // Raw counts: abs values = [100, 200], mean = 150
    assert!((result.mean - 150.0).abs() < 0.01, "mean should be 150.0 (raw), got {}", result.mean);
}

// ============================================================
// T025: E2E test: LITE format UDP delivery
// ============================================================
#[test]
fn test_rsam_e2e_lite() {
    let (listener, port) = bind_listener();

    let mut settings = make_settings(port);
    settings.interval = 0; // Trigger immediately
    settings.fwformat = "LITE".to_string();

    let mut mgr = RsamManager::new(&settings, HashMap::new()).unwrap();

    // Feed 200 samples (simulating 2 seconds at 100Hz)
    let samples: Vec<f64> = (1..=200).map(|i| i as f64).collect();
    let seg = make_segment("EHZ", "R6E01", samples);
    std::thread::sleep(Duration::from_millis(10));
    mgr.process_segment(&seg);

    let mut buf = [0u8; 4096];
    let (n, _) = listener.recv_from(&mut buf).unwrap();
    let received = std::str::from_utf8(&buf[..n]).unwrap();

    // Verify LITE format
    assert!(received.starts_with("stn:R6E01|ch:EHZ|mean:"), "LITE format wrong: {}", received);
    assert!(received.contains("|med:"), "Missing median: {}", received);
    assert!(received.contains("|min:"), "Missing min: {}", received);
    assert!(received.contains("|max:"), "Missing max: {}", received);

    // Verify values: abs values 1..200, mean = 100.5, min = 1, max = 200
    assert!(received.contains("|min:1|") || received.contains("|min:1."), "min wrong: {}", received);
}

// ============================================================
// T026: E2E test: JSON format
// ============================================================
#[test]
fn test_rsam_e2e_json() {
    let (listener, port) = bind_listener();

    let mut settings = make_settings(port);
    settings.interval = 0;
    settings.fwformat = "JSON".to_string();

    let mut mgr = RsamManager::new(&settings, HashMap::new()).unwrap();

    let seg = make_segment("EHZ", "R6E01", vec![10.0, 20.0, 30.0, 40.0, 50.0]);
    std::thread::sleep(Duration::from_millis(10));
    mgr.process_segment(&seg);

    let mut buf = [0u8; 4096];
    let (n, _) = listener.recv_from(&mut buf).unwrap();
    let received = std::str::from_utf8(&buf[..n]).unwrap();

    // Verify valid JSON
    let v: serde_json::Value = serde_json::from_str(received).expect("Invalid JSON");
    assert_eq!(v["station"], "R6E01");
    assert_eq!(v["channel"], "EHZ");
    assert_eq!(v["mean"], 30.0);
    assert_eq!(v["median"], 30.0);
    assert_eq!(v["min"], 10.0);
    assert_eq!(v["max"], 50.0);
}

// ============================================================
// T027: E2E test: deconvolution and filtering
// ============================================================
#[test]
fn test_rsam_e2e_deconv_filtering() {
    let (listener, port) = bind_listener();

    let mut settings = make_settings(port);
    settings.interval = 0;
    settings.channel = "EHZ".to_string();
    settings.deconvolve = true;
    settings.units = "VEL".to_string();
    settings.fwformat = "JSON".to_string();

    let mut sens_map = HashMap::new();
    sens_map.insert("EHZ".to_string(), 399000000.0);

    let mut mgr = RsamManager::new(&settings, sens_map).unwrap();

    // Send EHZ data
    let seg_ehz = make_segment("EHZ", "R6E01", vec![399000000.0, -798000000.0]);
    // Send EHN data (should be filtered out since channel="EHZ")
    let seg_ehn = make_segment("EHN", "R6E01", vec![999999999.0]);

    std::thread::sleep(Duration::from_millis(10));
    mgr.process_segment(&seg_ehz);
    mgr.process_segment(&seg_ehn);

    let mut buf = [0u8; 4096];
    let (n, _) = listener.recv_from(&mut buf).unwrap();
    let received = std::str::from_utf8(&buf[..n]).unwrap();

    let v: serde_json::Value = serde_json::from_str(received).expect("Invalid JSON");
    assert_eq!(v["channel"], "EHZ");
    // 399000000/399000000 = 1.0, 798000000/399000000 = 2.0, mean = 1.5
    let mean = v["mean"].as_f64().unwrap();
    assert!((mean - 1.5).abs() < 0.01, "mean should be 1.5 (deconvolved), got {}", mean);
}

// ============================================================
// T028: E2E test: multiple intervals
// ============================================================
#[test]
fn test_rsam_e2e_multiple_intervals() {
    let (listener, port) = bind_listener();

    let mut settings = make_settings(port);
    settings.interval = 1; // 1 second interval
    settings.fwformat = "JSON".to_string();

    let mut mgr = RsamManager::new(&settings, HashMap::new()).unwrap();

    // First batch of data
    let seg1 = make_segment("EHZ", "R6E01", vec![10.0, 20.0, 30.0]);

    mgr.process_segment(&seg1);

    // Wait for interval to elapse
    std::thread::sleep(Duration::from_millis(1100));

    // Second batch — different values — triggers first interval send
    let seg2 = make_segment("EHZ", "R6E01", vec![100.0, 200.0, 300.0]);
    mgr.process_segment(&seg2);

    // Read first packet
    let mut buf = [0u8; 4096];
    let (n, _) = listener.recv_from(&mut buf).unwrap();
    let pkt1 = std::str::from_utf8(&buf[..n]).unwrap().to_string();
    let v1: serde_json::Value = serde_json::from_str(&pkt1).expect("Invalid JSON pkt1");
    let mean1 = v1["mean"].as_f64().unwrap();

    // Wait for second interval
    std::thread::sleep(Duration::from_millis(1100));

    // Third batch — triggers second interval send
    let seg3 = make_segment("EHZ", "R6E01", vec![500.0, 600.0]);
    mgr.process_segment(&seg3);

    let (n2, _) = listener.recv_from(&mut buf).unwrap();
    let pkt2 = std::str::from_utf8(&buf[..n2]).unwrap().to_string();
    let v2: serde_json::Value = serde_json::from_str(&pkt2).expect("Invalid JSON pkt2");
    let mean2 = v2["mean"].as_f64().unwrap();

    // The two means should be different (buffer was reset between intervals)
    assert!(
        (mean1 - mean2).abs() > 1.0,
        "Two intervals should have different values: mean1={}, mean2={}",
        mean1,
        mean2
    );
}
