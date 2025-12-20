use chrono::Utc;
use rsudp_rust::trigger::{AlertEventType, TriggerConfig, TriggerManager};

#[tokio::test]
async fn test_sta_lta_trigger() {
    let mut tm = TriggerManager::new(TriggerConfig {
        sta_sec: 1.0,
        lta_sec: 10.0,
        threshold: 3.0,
        reset_threshold: 1.5,
    });

    let id = "TEST.CHZ";
    let ts = Utc::now();

    // 1. Warm up with baseline noise (no trigger)
    for _ in 0..1000 {
        assert!(tm.add_sample(id, 1.0, ts).is_none());
    }

    // 2. High amplitude (trigger)
    let mut alarm_event = None;
    for _ in 0..100 {
        if let Some(ev) = tm.add_sample(id, 100.0, ts) {
            alarm_event = Some(ev);
            break;
        }
    }

    let ev = alarm_event.expect("Should have triggered");
    matches!(ev.event_type, AlertEventType::Trigger);
    println!("Alarm message: {}", ev.message);

    // 3. Back to noise (reset)
    let mut reset_event = None;
    for _ in 0..2000 {
        if let Some(ev) = tm.add_sample(id, 1.0, ts) {
            reset_event = Some(ev);
            break;
        }
    }

    let ev = reset_event.expect("Should have reset");
    matches!(ev.event_type, AlertEventType::Reset);
    println!("Reset message: {}", ev.message);
}
