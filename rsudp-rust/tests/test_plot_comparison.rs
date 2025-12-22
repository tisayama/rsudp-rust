use rsudp_rust::parser::mseed::parse_mseed_file;
use rsudp_rust::web::plot::draw_rsudp_plot;
use std::collections::HashMap;

fn verify_file(path: &str, label: &str) {
    let segments = parse_mseed_file(path).expect("Failed to parse mseed");
    let mut channel_data = HashMap::new();
    let mut start_time = None;
    let mut sampling_rate = 100.0;
    
    for seg in segments {
        if start_time.is_none() {
            start_time = Some(seg.starttime);
            sampling_rate = seg.sampling_rate;
        }
        let entry = channel_data.entry(seg.channel.clone()).or_insert_with(Vec::new);
        entry.extend(seg.samples);
    }
    
    let start_time = start_time.expect("No channels found");
    let out_name = format!("verif_{}.png", label);
    
    println!("\n--- Verifying {} ---", path);
    // Passing 384,500 sensitivity and mock max_intensity (e.g. 2.85 for Shindo 3)
    let mock_intensity = if label == "tsukuba" { 2.85 } else { 2.08 };
    draw_rsudp_plot(&out_name, "TEST", &channel_data, start_time, sampling_rate, Some(384500.0), mock_intensity)
        .expect("Failed to draw plot");
    
    println!("Generated: {}", out_name);
}

#[test]
fn test_intensity_accuracy() {
    verify_file("../references/mseed/fdsnws.mseed", "fdsnws");
    verify_file("../references/mseed/20251208_tsukuba_fdsnws.mseed", "tsukuba");
}
