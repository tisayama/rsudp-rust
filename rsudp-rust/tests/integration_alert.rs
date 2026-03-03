use chrono::{Duration, Utc};
use rsudp_rust::trigger::{AlertEventType, TriggerConfig, TriggerManager};
use std::f64::consts::PI;

/// Integration test for windowed STA/LTA trigger.
/// Uses 1 Hz sine waves (within bandpass) since the windowed approach
/// creates fresh filters each evaluation, rejecting DC/constant values.
#[tokio::test]
async fn test_sta_lta_trigger() {
    let mut tm = TriggerManager::new(TriggerConfig {
        sta_sec: 1.0,
        lta_sec: 10.0,
        threshold: 3.0,
        reset_threshold: 1.5,
        highpass: 0.1,
        lowpass: 5.0,
        target_channel: "HZ".to_string(),
        duration: 0.0,
    });

    let id = "TEST.EHZ";
    let base_ts = Utc::now();
    let sensitivity = 1.0;
    let sps = 100.0;
    let nlta = (10.0 * sps) as usize; // 1000

    let mut sample_idx: usize = 0;

    // 1. Fill buffer with baseline noise (1 Hz sine, amplitude 1.0)
    for _ in 0..(nlta + 200) {
        let val = 1.0 * (2.0 * PI * 1.0 * sample_idx as f64 / sps).sin();
        let ts = base_ts + Duration::milliseconds(sample_idx as i64 * 10);
        tm.add_sample(id, val, ts, sensitivity);
        sample_idx += 1;
    }

    // 2. High amplitude sine wave (trigger)
    let mut alarm_event = None;
    for _ in 0..500 {
        let val = 100.0 * (2.0 * PI * 1.0 * sample_idx as f64 / sps).sin();
        let ts = base_ts + Duration::milliseconds(sample_idx as i64 * 10);
        if let Some(ev) = tm.add_sample(id, val, ts, sensitivity) {
            if ev.event_type == AlertEventType::Trigger {
                alarm_event = Some(ev);
                break;
            }
        }
        sample_idx += 1;
    }

    let ev = alarm_event.expect("Should have triggered");
    assert_eq!(ev.event_type, AlertEventType::Trigger);
    println!("Alarm triggered at ratio: {:.4}", ev.ratio);

    // 3. Back to noise (reset) — need enough samples for the entire buffer
    // to transition back to noise-only content
    let mut reset_event = None;
    for _ in 0..(nlta + 1000) {
        let val = 1.0 * (2.0 * PI * 1.0 * sample_idx as f64 / sps).sin();
        let ts = base_ts + Duration::milliseconds(sample_idx as i64 * 10);
        if let Some(ev) = tm.add_sample(id, val, ts, sensitivity) {
            if ev.event_type == AlertEventType::Reset {
                reset_event = Some(ev);
                break;
            }
        }
        sample_idx += 1;
    }

    let ev = reset_event.expect("Should have reset");
    assert_eq!(ev.event_type, AlertEventType::Reset);
    println!("Reset at ratio: {:.4}, max: {:.4}", ev.ratio, ev.max_ratio);
}
