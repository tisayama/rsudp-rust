use rsudp_rust::trigger::{AlertManager, AlertConfig, AlertEventType};
use chrono::Utc;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_integration_alert_lifecycle() {
    let (tx, mut rx) = mpsc::channel(100);
    let config = AlertConfig {
        sta_seconds: 1.0,
        lta_seconds: 5.0,
        threshold: 2.0,
        reset_threshold: 1.5,
        min_duration: 0.0,
        channel_id: "SHZ".to_string(),
        filter_config: None,
        sample_rate: 10.0,
    };
    
    let mut manager = AlertManager::new(config, tx);
    let start_time = Utc::now();
    
    // 1. Warm-up Phase (50 samples for 5s @ 10Hz)
    for i in 0..50 {
        let ts = start_time + chrono::Duration::milliseconds(i * 100);
        manager.process_sample(1.0, ts).await.unwrap();
    }
    
    // 2. Monitoring Phase - No Alarm
    for i in 50..60 {
        let ts = start_time + chrono::Duration::milliseconds(i * 100);
        manager.process_sample(1.1, ts).await.unwrap();
    }
    assert!(rx.try_recv().is_err());
    
    // 3. Seismic Event - Alarm Trigger
    // Inject a strong signal
    for i in 60..70 {
        let ts = start_time + chrono::Duration::milliseconds(i * 100);
        manager.process_sample(10.0, ts).await.unwrap();
    }
    
    let alarm_event = rx.recv().await.expect("Should receive ALARM");
    assert_eq!(alarm_event.event_type, AlertEventType::Alarm);
    assert_eq!(alarm_event.channel_id, "SHZ");
    
    // 4. Signal Subsides - Reset Trigger
    for i in 70..150 {
        let ts = start_time + chrono::Duration::milliseconds(i * 100);
        manager.process_sample(0.5, ts).await.unwrap();
    }
    
    let reset_event = rx.recv().await.expect("Should receive RESET");
    assert_eq!(reset_event.event_type, AlertEventType::Reset);
    assert!(reset_event.max_ratio.is_some());
    assert!(reset_event.max_ratio.unwrap() > 2.0);
}

#[tokio::test]
async fn test_integration_alert_gap_recovery() {
    let (tx, mut rx) = mpsc::channel(100);
    let config = AlertConfig {
        sta_seconds: 1.0,
        lta_seconds: 5.0,
        threshold: 2.0,
        reset_threshold: 1.5,
        min_duration: 0.0,
        channel_id: "SHZ".to_string(),
        filter_config: None,
        sample_rate: 10.0,
    };
    
    let mut manager = AlertManager::new(config, tx);
    let start_time = Utc::now();
    
    // Warm up
    for i in 0..50 {
        let ts = start_time + chrono::Duration::milliseconds(i * 100);
        manager.process_sample(1.0, ts).await.unwrap();
    }
    
    // Inject a gap (10 seconds)
    let gap_ts = start_time + chrono::Duration::milliseconds(15000);
    manager.process_sample(1.0, gap_ts).await.unwrap();
    
    // Should be back in WarmingUp
    // If we inject a spike now, it should NOT trigger because it's warming up
    manager.process_sample(100.0, gap_ts + chrono::Duration::milliseconds(100)).await.unwrap();
    assert!(rx.try_recv().is_err());
}
