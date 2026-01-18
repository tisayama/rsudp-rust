use std::process::{Child, Command};
use std::net::UdpSocket;
use std::time::{Duration, Instant};
use std::thread;
use std::io::Read;
use std::fs;
use tempfile::tempdir;
use regex::Regex;

// T002: Helper to find a free UDP port
fn get_free_port() -> u16 {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind to free port");
    socket.local_addr().expect("Failed to get local addr").port()
}

// T003: BackgroundProcess struct
struct BackgroundProcess {
    child: Child,
    _name: String,
}

impl BackgroundProcess {
    fn new(mut cmd: Command, name: &str) -> Self {
        let child = cmd.spawn().expect(&format!("Failed to spawn {}", name));
        Self {
            child,
            _name: name.to_string(),
        }
    }
}

impl Drop for BackgroundProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn test_e2e_alert_triggering() {
    let udp_port = get_free_port();
    let web_port = get_free_port();
    let temp_dir = tempdir().unwrap();
    let log_path = temp_dir.path().join("rsudp.log");
    // The app expects alerts directory inside its output_dir
    let alerts_dir = temp_dir.path().join("alerts");
    fs::create_dir_all(&alerts_dir).unwrap();

    println!("Starting E2E test on UDP port {}, Web port {}, Temp dir {:?}", udp_port, web_port, temp_dir.path());

    // T005: Spawn rsudp-rust
    let log_file = fs::File::create(&log_path).expect("Failed to create log file");
    
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--bin", "rsudp-rust", "--"])
       .arg("--udp-port").arg(udp_port.to_string())
       .arg("--web-port").arg(web_port.to_string())
       .arg("--station").arg("R6E01")
       .arg("--network").arg("AM")
       .arg("--output-dir").arg(temp_dir.path())
       .arg("--save-pct").arg("0.1")
       .env("RUST_LOG", "rsudp_rust::pipeline=info,rsudp_rust::trigger=info,rsudp_rust::web::alerts=info")
       .stdout(log_file.try_clone().unwrap())
       .stderr(log_file);

    let _rsudp_proc = BackgroundProcess::new(cmd, "rsudp-rust");

    // Wait for rsudp to start up
    thread::sleep(Duration::from_secs(5));

    // T006: Spawn streamer (100x speed)
    let mseed_path = "../references/mseed/fdsnws.mseed";
    let mut streamer_cmd = Command::new("cargo");
    streamer_cmd.args(&["run", "--bin", "streamer", "--"])
        .arg("--file").arg(mseed_path)
        .arg("--addr").arg(format!("127.0.0.1:{}", udp_port))
        .arg("--speed").arg("100.0");

    let _streamer_proc = BackgroundProcess::new(streamer_cmd, "streamer");

    // T007: Assertions
    let start = Instant::now();
    let timeout = Duration::from_secs(120); // 2 minutes max
    let mut alarm_found = false;
    let mut snapshot_saved = false;
    let mut png_found = false;
    
    let alarm_regex = Regex::new(r"ALARM!").unwrap();
    let saved_regex = Regex::new(r"Snapshot saved successfully").unwrap();
    let error_regex = Regex::new(r"Failed to generate snapshot").unwrap();

    while start.elapsed() < timeout {
        if let Ok(mut file) = fs::File::open(&log_path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap_or_default();
            
            if !alarm_found && alarm_regex.is_match(&contents) {
                println!("Found ALARM in log!");
                alarm_found = true;
            }
            
            if alarm_found && !snapshot_saved && saved_regex.is_match(&contents) {
                println!("Snapshot save confirmed in log!");
                snapshot_saved = true;
            }
            
            if error_regex.is_match(&contents) {
                println!("--- RSUDP LOG DUMP ---");
                println!("{}", contents);
                panic!("Snapshot generation failed according to logs");
            }
        }

        // Search recursively for PNG in temp_dir
        if alarm_found && !png_found {
            fn find_png(dir: &std::path::Path) -> Option<std::path::PathBuf> {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if path.is_dir() {
                                if let Some(found) = find_png(&path) { return Some(found); }
                            } else if path.extension().and_then(|s| s.to_str()) == Some("png") {
                                return Some(path);
                            }
                        }
                    }
                }
                None
            }
            
            if let Some(path) = find_png(temp_dir.path()) {
                println!("Found alert image: {:?}", path);
                png_found = true;
            }
        }

        if alarm_found && snapshot_saved && png_found {
            break;
        }

        thread::sleep(Duration::from_millis(500));
    }

    if !alarm_found {
        println!("--- RSUDP LOG DUMP ---");
        if let Ok(c) = fs::read_to_string(&log_path) { println!("{}", c); }
        panic!("Timed out waiting for ALARM log message");
    }

    if !png_found {
        println!("--- RSUDP LOG DUMP ---");
        if let Ok(c) = fs::read_to_string(&log_path) { println!("{}", c); }
        panic!("Alert image file was not generated in time ({}s)", start.elapsed().as_secs());
    }
    
    println!("E2E Test Passed in {}s!", start.elapsed().as_secs());
}