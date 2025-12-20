use chrono::Utc;
use rsudp_rust::intensity::{IntensityConfig, IntensityManager};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;

#[test]
fn test_intensity_with_verified_samples() {
    let csv_path =
        "/home/tisayama/Development/rustrsudp_speckit/rsudp-rust/target/tmp/decoded_samples.csv";
    let file = File::open(csv_path).expect("Failed to open verified samples CSV");
    let reader = BufReader::new(file);

    let mut manager = IntensityManager::new(IntensityConfig {
        channels: vec!["ENE".to_string(), "ENN".to_string(), "ENZ".to_string()],
        sample_rate: 100.0,
        sensitivities: vec![1.0 / 384500.0, 1.0 / 384500.0, 1.0 / 384500.0],
    });

    let mut lines = reader.lines();
    let _header = lines.next(); // Skip header

    let mut all_ene = Vec::new();
    let mut all_enn = Vec::new();
    let mut all_enz = Vec::new();

    for line in lines {
        let l = line.expect("Failed to read line");
        let parts: Vec<f64> = l
            .split(',')
            .map(|s| s.parse().expect("Invalid float in CSV"))
            .collect();
        all_ene.push(parts[0]);
        all_enn.push(parts[1]);
        all_enz.push(parts[2]);
    }

    let mut results = Vec::new();
    for i in 0..(all_ene.len() / 100) {
        let mut map = HashMap::new();
        map.insert("ENE".to_string(), all_ene[i * 100..(i + 1) * 100].to_vec());
        map.insert("ENN".to_string(), all_enn[i * 100..(i + 1) * 100].to_vec());
        map.insert("ENZ".to_string(), all_enz[i * 100..(i + 1) * 100].to_vec());
        manager.add_samples(map, Utc::now());
        results.extend(manager.get_results());
    }

    if !results.is_empty() {
        for res in results {
            println!(
                "[{}] 計測震度: {:.2} ({})",
                res.timestamp, res.intensity, res.shindo_class
            );
        }
    }
}

#[test]
fn test_integration_intensity_fdsnws_mseed() {
    let mseed_path = "/home/tisayama/Development/rustrsudp_speckit/references/mseed/fdsnws.mseed";

    let output = Command::new("target/debug/rsudp-rust")
        .arg("--file")
        .arg(mseed_path)
        .arg("--channels")
        .arg("ENE,ENN,ENZ")
        .output()
        .expect("Failed to execute rsudp-rust");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);

    // After our fixes, we expect at least one intensity result to be printed
    assert!(
        stdout.contains("計測震度") || stderr.contains("計測震度"),
        "Intensity calculation was not triggered"
    );
}
