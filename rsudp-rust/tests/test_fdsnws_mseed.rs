use chrono::Duration;
use rsudp_rust::parser::mseed::parse_mseed_file;
use rsudp_rust::trigger::{AlertEventType, TriggerConfig, TriggerManager};
use std::path::Path;

/// Verify windowed STA/LTA on fdsnws.mseed matches Python rsudp results.
/// Python rsudp reference:
///   ALARM at t=85.8s  (ratio=1.1499), RESET at t=138.8s (ratio=0.4937, max=4.9234)
///   ALARM at t=166.0s (ratio=1.1035), RESET at t=334.2s (ratio=0.4998, max=1.7093)
#[test]
fn test_fdsnws_mseed() {
    let path = "../references/mseed/fdsnws.mseed";
    if !Path::new(path).exists() {
        eprintln!("Skipping: {} not found", path);
        return;
    }

    let segments = parse_mseed_file(path).expect("Should parse fdsnws.mseed");
    let mut ehz_segments: Vec<_> = segments
        .into_iter()
        .filter(|s| s.channel == "EHZ")
        .collect();
    assert!(!ehz_segments.is_empty(), "No EHZ channel in fdsnws.mseed");

    ehz_segments.sort_by_key(|s| s.starttime);

    let base_time = ehz_segments[0].starttime;
    let sps = ehz_segments[0].sampling_rate;
    let all_samples: Vec<f64> = ehz_segments
        .iter()
        .flat_map(|s| s.samples.iter().copied())
        .collect();

    println!(
        "EHZ: {} samples ({:.1}s), sps={}, base_time={}",
        all_samples.len(),
        all_samples.len() as f64 / sps,
        sps,
        base_time
    );

    let mut tm = TriggerManager::new(TriggerConfig {
        sta_sec: 6.0,
        lta_sec: 30.0,
        threshold: 1.1,
        reset_threshold: 0.5,
        highpass: 0.1,
        lowpass: 2.0,
        target_channel: "HZ".to_string(),
        duration: 0.0,
    });

    let nslc = ehz_segments[0].nslc();
    let mut alarm_events = Vec::new();
    let mut reset_events = Vec::new();

    for (i, &sample) in all_samples.iter().enumerate() {
        let micros = (i as f64 / sps * 1_000_000.0) as i64;
        let ts = base_time + Duration::microseconds(micros);

        if let Some(ev) = tm.add_sample(&nslc, sample, ts, 1.0) {
            let t_sec = (ev.timestamp - base_time).num_milliseconds() as f64 / 1000.0;
            match ev.event_type {
                AlertEventType::Trigger => {
                    println!(
                        "ALARM at t={:.1}s {} (ratio: {:.4})",
                        t_sec, ev.timestamp, ev.ratio
                    );
                    alarm_events.push(ev);
                }
                AlertEventType::Reset => {
                    println!(
                        "RESET at t={:.1}s {} (ratio: {:.4}, max: {:.4})",
                        t_sec, ev.timestamp, ev.ratio, ev.max_ratio
                    );
                    reset_events.push(ev);
                }
                AlertEventType::Status => {}
            }
        }
    }

    println!();
    println!("--- fdsnws.mseed Summary ---");
    println!("  ALARM events: {}", alarm_events.len());
    println!("  RESET events: {}", reset_events.len());

    // Compare with Python rsudp reference
    // Python: ALARM=2, RESET=2
    assert_eq!(alarm_events.len(), 2, "Expected 2 ALARM events (matching Python rsudp)");
    assert_eq!(reset_events.len(), 2, "Expected 2 RESET events (matching Python rsudp)");

    // First ALARM→RESET pair
    let a1_t = (alarm_events[0].timestamp - base_time).num_milliseconds() as f64 / 1000.0;
    let r1_t = (reset_events[0].timestamp - base_time).num_milliseconds() as f64 / 1000.0;
    println!("  Pair 1: ALARM t={:.1}s ratio={:.4}, RESET t={:.1}s ratio={:.4} max={:.4}",
        a1_t, alarm_events[0].ratio, r1_t, reset_events[0].ratio, reset_events[0].max_ratio);

    // Second ALARM→RESET pair
    let a2_t = (alarm_events[1].timestamp - base_time).num_milliseconds() as f64 / 1000.0;
    let r2_t = (reset_events[1].timestamp - base_time).num_milliseconds() as f64 / 1000.0;
    println!("  Pair 2: ALARM t={:.1}s ratio={:.4}, RESET t={:.1}s ratio={:.4} max={:.4}",
        a2_t, alarm_events[1].ratio, r2_t, reset_events[1].ratio, reset_events[1].max_ratio);

    // Python reference: ALARM at 85.8s, RESET at 138.8s (±1s for packet boundary tolerance)
    assert!((a1_t - 85.8).abs() < 1.0,
        "First ALARM should be at ~85.8s, got {:.1}s", a1_t);
    assert!((r1_t - 138.8).abs() < 1.0,
        "First RESET should be at ~138.8s, got {:.1}s", r1_t);

    // Python reference: ALARM at 166.0s, RESET at 334.2s
    assert!((a2_t - 166.0).abs() < 1.0,
        "Second ALARM should be at ~166.0s, got {:.1}s", a2_t);
    assert!((r2_t - 334.2).abs() < 1.0,
        "Second RESET should be at ~334.2s, got {:.1}s", r2_t);

    // Max ratio comparison (Python: 4.9234 and 1.7093)
    assert!((reset_events[0].max_ratio - 4.9234).abs() < 0.05,
        "First max_ratio should be ~4.9234, got {:.4}", reset_events[0].max_ratio);
    assert!((reset_events[1].max_ratio - 1.7093).abs() < 0.05,
        "Second max_ratio should be ~1.7093, got {:.4}", reset_events[1].max_ratio);
}
