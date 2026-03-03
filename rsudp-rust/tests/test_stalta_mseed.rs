use chrono::Duration;
use rsudp_rust::parser::mseed::parse_mseed_file;
use rsudp_rust::trigger::{AlertEventType, TriggerConfig, TriggerManager};
use std::path::Path;

/// Verify windowed STA/LTA against shindo0.mseed reference data.
/// Expected: ALARM fires, RESET fires, duration ≈ 160s (windowed approach tracks coda),
/// max ratio clearly above threshold.
#[test]
fn test_streaming_stalta_shindo0() {
    let path = "../references/mseed/shindo0.mseed";
    if !Path::new(path).exists() {
        eprintln!("Skipping: {} not found", path);
        return;
    }

    let segments = parse_mseed_file(path).expect("Should parse shindo0.mseed");
    let mut ehz_segments: Vec<_> = segments
        .into_iter()
        .filter(|s| s.channel == "EHZ")
        .collect();
    assert!(!ehz_segments.is_empty(), "No EHZ channel in shindo0.mseed");

    // Sort segments by starttime to ensure chronological order
    ehz_segments.sort_by_key(|s| s.starttime);

    // Merge all EHZ samples into a single continuous stream
    // Use the first segment's starttime as the global base
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
        // Compute timestamp from global base (matching Python: starttime + i/SPS)
        let micros = (i as f64 / sps * 1_000_000.0) as i64;
        let ts = base_time + Duration::microseconds(micros);

        if let Some(ev) = tm.add_sample(&nslc, sample, ts, 1.0) {
            match ev.event_type {
                AlertEventType::Trigger => {
                    println!(
                        "ALARM at {} (ratio: {:.4})",
                        ev.timestamp, ev.ratio
                    );
                    alarm_events.push(ev);
                }
                AlertEventType::Reset => {
                    println!(
                        "RESET at {} (ratio: {:.4}, max: {:.4})",
                        ev.timestamp, ev.ratio, ev.max_ratio
                    );
                    reset_events.push(ev);
                }
                AlertEventType::Status => {}
            }
        }
    }

    // Must have at least one ALARM and one RESET
    assert!(
        !alarm_events.is_empty(),
        "Expected at least one ALARM event"
    );
    assert!(
        !reset_events.is_empty(),
        "Expected at least one RESET event"
    );

    // ALARM→RESET duration: windowed approach tracks earthquake coda faithfully
    // Duration is ~160s (longer than streaming EMA's ~72s because coda energy
    // persists in the evaluation window)
    let alarm_ts = alarm_events[0].timestamp;
    let reset_ts = reset_events[0].timestamp;
    let duration_secs = (reset_ts - alarm_ts).num_milliseconds() as f64 / 1000.0;
    println!("ALARM→RESET duration: {:.1}s", duration_secs);
    assert!(
        duration_secs > 50.0 && duration_secs < 250.0,
        "ALARM→RESET should be 50-250s, got {:.1}s",
        duration_secs
    );

    // Max ratio should clearly exceed threshold (1.1), confirming earthquake detection
    let max_ratio = reset_events[0].max_ratio;
    println!("Max ratio: {:.4}", max_ratio);
    assert!(
        max_ratio > 2.0,
        "Max ratio should clearly exceed threshold, got {:.4}",
        max_ratio
    );
}
