use rsudp_rust::parser::mseed::parse_mseed_file;
use rsudp_rust::web::plot::draw_rsudp_plot;
use std::collections::HashMap;

#[test]
fn test_generate_rust_plot() {
    let path = "../references/mseed/fdsnws.mseed";
    let segments = parse_mseed_file(path).expect("Failed to parse mseed");
    
    // Select and merge all EHZ channel segments
    let mut full_samples: Vec<f64> = Vec::new();
    let mut start_time = None;
    let mut sampling_rate = 100.0;
    
    for seg in segments {
        if seg.channel == "EHZ" {
            if start_time.is_none() {
                start_time = Some(seg.starttime);
                sampling_rate = seg.sampling_rate;
            }
            full_samples.extend(seg.samples);
        }
    }
    
    let start_time = start_time.expect("EHZ not found");

    // Take first 300 seconds
    let n_samples = (sampling_rate * 300.0) as usize;
    let data = if full_samples.len() > n_samples {
        full_samples[0..n_samples].to_vec()
    } else {
        full_samples
    };
    
    println!("Data length: {} samples", data.len());
    println!("Expected duration: {} seconds", data.len() as f64 / sampling_rate);

    let mut channel_data = HashMap::new();
    channel_data.insert("EHZ".to_string(), data);
    
    // Using R6E01 as station name (from file)
    draw_rsudp_plot("rust_comparison_300s.png", "R6E01", &channel_data, start_time, sampling_rate, None)
        .expect("Failed to draw plot");
    
    println!("Rust comparison plot saved to rust_comparison_300s.png");
}
