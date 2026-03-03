use chrono::Duration;
use rsudp_rust::parser::mseed::parse_mseed_file;
use rsudp_rust::trigger::{AlertEventType, TriggerConfig, TriggerManager};
use std::path::Path;

/// Feed normal.mseed (non-earthquake ambient data) through TriggerManager
/// and check whether false alarms fire.
/// Expected: NO ALARM events during normal seismic background noise.
#[test]
fn test_normal_mseed_no_false_alarm() {
    let path = "../references/mseed/normal.mseed";
    if !Path::new(path).exists() {
        eprintln!("Skipping: {} not found", path);
        return;
    }

    let segments = parse_mseed_file(path).expect("Should parse normal.mseed");
    let mut ehz_segments: Vec<_> = segments
        .into_iter()
        .filter(|s| s.channel == "EHZ")
        .collect();

    if ehz_segments.is_empty() {
        eprintln!("No EHZ channel in normal.mseed, skipping");
        return;
    }

    // Sort segments by starttime to ensure chronological order
    ehz_segments.sort_by_key(|s| s.starttime);

    let base_time = ehz_segments[0].starttime;
    let sps = ehz_segments[0].sampling_rate;
    let all_samples: Vec<f64> = ehz_segments
        .iter()
        .flat_map(|s| s.samples.iter().copied())
        .collect();

    println!("=== normal.mseed ===");
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
    let mut alarm_count = 0u64;
    let mut reset_count = 0u64;
    let mut status_count = 0u64;
    let mut max_ratio_seen = 0.0f64;
    let mut alarm_events = Vec::new();

    for (i, &sample) in all_samples.iter().enumerate() {
        let micros = (i as f64 / sps * 1_000_000.0) as i64;
        let ts = base_time + Duration::microseconds(micros);

        if let Some(ev) = tm.add_sample(&nslc, sample, ts, 1.0) {
            if ev.ratio > max_ratio_seen {
                max_ratio_seen = ev.ratio;
            }
            match ev.event_type {
                AlertEventType::Trigger => {
                    alarm_count += 1;
                    println!(
                        "  ALARM at {} (sample {}, ratio: {:.4})",
                        ev.timestamp, i, ev.ratio
                    );
                    alarm_events.push((ev.timestamp, ev.ratio));
                }
                AlertEventType::Reset => {
                    reset_count += 1;
                    println!(
                        "  RESET at {} (sample {}, ratio: {:.4}, max: {:.4})",
                        ev.timestamp, i, ev.ratio, ev.max_ratio
                    );
                }
                AlertEventType::Status => {
                    status_count += 1;
                    println!(
                        "  STATUS at {} (sample {}, ratio: {:.4})",
                        ev.timestamp, i, ev.ratio
                    );
                }
            }
        }
    }

    println!();
    println!("--- normal.mseed Summary ---");
    println!("  ALARM events: {}", alarm_count);
    println!("  RESET events: {}", reset_count);
    println!("  STATUS events: {}", status_count);
    println!("  Max ratio seen: {:.4}", max_ratio_seen);
    println!();

    // Primary assertion: no false alarms on normal ambient data
    assert_eq!(
        alarm_count, 0,
        "Expected ZERO ALARM events on normal ambient data, but got {}. \
         False alarm timestamps: {:?}",
        alarm_count, alarm_events
    );
}

/// Feed normal2.mseed (non-earthquake ambient data) through TriggerManager
/// and check whether false alarms fire.
/// Expected: NO ALARM events during normal seismic background noise.
#[test]
fn test_normal2_mseed_no_false_alarm() {
    let path = "../references/mseed/normal2.mseed";
    if !Path::new(path).exists() {
        eprintln!("Skipping: {} not found", path);
        return;
    }

    let segments = parse_mseed_file(path).expect("Should parse normal2.mseed");
    let mut ehz_segments: Vec<_> = segments
        .into_iter()
        .filter(|s| s.channel == "EHZ")
        .collect();

    if ehz_segments.is_empty() {
        eprintln!("No EHZ channel in normal2.mseed, skipping");
        return;
    }

    // Sort segments by starttime to ensure chronological order
    ehz_segments.sort_by_key(|s| s.starttime);

    let base_time = ehz_segments[0].starttime;
    let sps = ehz_segments[0].sampling_rate;
    let all_samples: Vec<f64> = ehz_segments
        .iter()
        .flat_map(|s| s.samples.iter().copied())
        .collect();

    println!("=== normal2.mseed ===");
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
    let mut alarm_count = 0u64;
    let mut reset_count = 0u64;
    let mut status_count = 0u64;
    let mut max_ratio_seen = 0.0f64;
    let mut alarm_events = Vec::new();

    for (i, &sample) in all_samples.iter().enumerate() {
        let micros = (i as f64 / sps * 1_000_000.0) as i64;
        let ts = base_time + Duration::microseconds(micros);

        if let Some(ev) = tm.add_sample(&nslc, sample, ts, 1.0) {
            if ev.ratio > max_ratio_seen {
                max_ratio_seen = ev.ratio;
            }
            match ev.event_type {
                AlertEventType::Trigger => {
                    alarm_count += 1;
                    println!(
                        "  ALARM at {} (sample {}, ratio: {:.4})",
                        ev.timestamp, i, ev.ratio
                    );
                    alarm_events.push((ev.timestamp, ev.ratio));
                }
                AlertEventType::Reset => {
                    reset_count += 1;
                    println!(
                        "  RESET at {} (sample {}, ratio: {:.4}, max: {:.4})",
                        ev.timestamp, i, ev.ratio, ev.max_ratio
                    );
                }
                AlertEventType::Status => {
                    status_count += 1;
                    println!(
                        "  STATUS at {} (sample {}, ratio: {:.4})",
                        ev.timestamp, i, ev.ratio
                    );
                }
            }
        }
    }

    println!();
    println!("--- normal2.mseed Summary ---");
    println!("  ALARM events: {}", alarm_count);
    println!("  RESET events: {}", reset_count);
    println!("  STATUS events: {}", status_count);
    println!("  Max ratio seen: {:.4}", max_ratio_seen);
    println!();

    // Primary assertion: no false alarms on normal ambient data
    assert_eq!(
        alarm_count, 0,
        "Expected ZERO ALARM events on normal ambient data, but got {}. \
         False alarm timestamps: {:?}",
        alarm_count, alarm_events
    );
}
