use chrono::{DateTime, Utc};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub id: Uuid,
    pub channel: String,
    pub trigger_time: DateTime<Utc>,
    pub reset_time: Option<DateTime<Utc>>,
    pub max_ratio: f64,
    pub snapshot_path: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSettings {
    pub audio_enabled: bool,
    pub email_enabled: bool,
    pub flash_enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub email_recipient: String,
    pub save_pct: f64,
}

impl Default for AlertSettings {
    fn default() -> Self {
        Self {
            audio_enabled: true,
            email_enabled: false,
            flash_enabled: true,
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            smtp_user: "".to_string(),
            smtp_pass: "".to_string(),
            email_recipient: "".to_string(),
            save_pct: 0.7,
        }
    }
}

pub fn format_shindo_message(shindo_class: &str) -> String {
    if shindo_class == "0" {
        "揺れを検出できませんでした".to_string()
    } else {
        let display_class = match shindo_class {
            "5-" => "5弱",
            "5+" => "5強",
            "6-" => "6弱",
            "6+" => "6強",
            _ => shindo_class,
        };
        format!("震度 {}相当の揺れを検出しました", display_class)
    }
}

pub fn send_trigger_email(
    settings: &AlertSettings,
    channel: &str,
    time: DateTime<Utc>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !settings.email_enabled {
        return Ok(());
    }

    let email = Message::builder()
        .from("RSRUSTUDP <noreply@example.com>".parse()?)
        .to(settings.email_recipient.parse()?)
        .subject(format!("ALERT TRIGGERED: {}", channel))
        .body(format!(
            "Seismic alert triggered on channel {}\nTime: {}\n",
            channel,
            time,
        ))?;

    let creds = Credentials::new(settings.smtp_user.clone(), settings.smtp_pass.clone());
    let mailer = SmtpTransport::relay(&settings.smtp_host)?
        .port(settings.smtp_port)
        .credentials(creds)
        .build();

    mailer.send(&email)?;
    Ok(())
}

pub fn send_reset_email(
    settings: &AlertSettings,
    channel: &str,
    trigger_time: DateTime<Utc>,
    reset_time: DateTime<Utc>,
    max_ratio: f64,
    snapshot_url: Option<&str>,
    intensity_message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !settings.email_enabled {
        return Ok(());
    }

    let mut body = format!(
        "Seismic alert RESET on channel {}\n\nSummary: {}\nTriggered: {}\nReset: {}\nMax STA/LTA Ratio: {:.2}\n",
        channel,
        intensity_message,
        trigger_time,
        reset_time,
        max_ratio,
    );

    if let Some(url) = snapshot_url {
        body.push_str(&format!("\nWaveform Snapshot: {}", url));
    }

    let email = Message::builder()
        .from("RSRUSTUDP <noreply@example.com>".parse()?)
        .to(settings.email_recipient.parse()?)
        .subject(format!("ALERT SUMMARY: {}", channel))
        .body(body)?;

    let creds = Credentials::new(settings.smtp_user.clone(), settings.smtp_pass.clone());
    let mailer = SmtpTransport::relay(&settings.smtp_host)?
        .port(settings.smtp_port)
        .credentials(creds)
        .build();

    mailer.send(&email)?;
    Ok(())
}

use std::path::{Path, PathBuf};

/// Capture a screenshot via the Playwright capture service.
///
/// Sends a CaptureRequest to the capture service and writes the returned PNG
/// to `{output_dir}/alerts/{uuid}.png`. Returns the file path on success, or
/// None on any failure (timeout, connection error, HTTP error).
pub async fn capture_screenshot(
    service_url: &str,
    timeout_seconds: u64,
    station: &str,
    channels: &[String],
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    intensity_class: &str,
    intensity_value: f64,
    backend_url: &str,
    output_dir: &Path,
) -> Option<PathBuf> {
    use tracing::warn;

    let alert_id = Uuid::new_v4();
    let filename = format!("{}.png", alert_id);
    let alerts_dir = output_dir.join("alerts");

    let request_body = serde_json::json!({
        "station": station,
        "channels": channels,
        "start_time": start_time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        "end_time": end_time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        "intensity_class": intensity_class,
        "intensity_value": intensity_value,
        "backend_url": backend_url,
        "width": 1000,
        "height": 500 * channels.len(),
    });

    let url = format!("{}/capture", service_url.trim_end_matches('/'));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_seconds))
        .build()
        .unwrap_or_default();

    let response = match client.post(&url).json(&request_body).send().await {
        Ok(resp) => resp,
        Err(e) => {
            warn!("Capture service request failed: {}", e);
            return None;
        }
    };

    if !response.status().is_success() {
        warn!(
            "Capture service returned HTTP {}: {}",
            response.status(),
            response.text().await.unwrap_or_default()
        );
        return None;
    }

    let png_bytes = match response.bytes().await {
        Ok(b) => b,
        Err(e) => {
            warn!("Failed to read capture response body: {}", e);
            return None;
        }
    };

    if let Err(e) = std::fs::create_dir_all(&alerts_dir) {
        warn!("Failed to create alerts directory: {}", e);
        return None;
    }

    let path = alerts_dir.join(&filename);
    if let Err(e) = std::fs::write(&path, &png_bytes) {
        warn!("Failed to write capture PNG to {}: {}", path.display(), e);
        return None;
    }

    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// T006: Test capture_screenshot success — mock server returns PNG, verify file written
    #[tokio::test]
    async fn test_capture_screenshot_success() {
        // Start a mock HTTP server that returns a fake PNG
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = std::io::Read::read(&mut stream, &mut buf);
                let fake_png = b"\x89PNG\r\n\x1a\nfake-png-data";
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\n\r\n",
                    fake_png.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(fake_png);
            }
        });

        let tmp_dir = tempfile::tempdir().unwrap();
        let result = capture_screenshot(
            &format!("http://127.0.0.1:{}", addr.port()),
            5,
            "AM.R6E01",
            &["EHZ".to_string(), "EHN".to_string()],
            Utc::now() - chrono::Duration::seconds(60),
            Utc::now(),
            "3",
            2.85,
            "http://localhost:8080",
            tmp_dir.path(),
        )
        .await;

        handle.join().unwrap();

        assert!(result.is_some(), "capture_screenshot should return Some(path)");
        let path = result.unwrap();
        assert!(path.exists(), "PNG file should exist at {:?}", path);
        assert!(path.to_str().unwrap().contains("alerts/"));
        let content = std::fs::read(&path).unwrap();
        assert!(content.starts_with(b"\x89PNG"), "File should start with PNG magic bytes");
    }

    /// T006: Test capture_screenshot timeout — mock server never responds, returns None
    #[tokio::test]
    async fn test_capture_screenshot_timeout() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        // Accept connection but never respond (causes timeout)
        let handle = std::thread::spawn(move || {
            if let Ok((_stream, _)) = listener.accept() {
                std::thread::sleep(std::time::Duration::from_secs(10));
            }
        });

        let tmp_dir = tempfile::tempdir().unwrap();
        let result = capture_screenshot(
            &format!("http://127.0.0.1:{}", addr.port()),
            1, // 1 second timeout
            "AM.R6E01",
            &["EHZ".to_string()],
            Utc::now() - chrono::Duration::seconds(60),
            Utc::now(),
            "0",
            0.0,
            "http://localhost:8080",
            tmp_dir.path(),
        )
        .await;

        assert!(result.is_none(), "capture_screenshot should return None on timeout");

        drop(handle); // Don't join — let the thread die
    }

    /// T006: Test capture_screenshot with connection refused — returns None
    #[tokio::test]
    async fn test_capture_screenshot_connection_refused() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let result = capture_screenshot(
            "http://127.0.0.1:1", // Port 1 should refuse connection
            2,
            "AM.R6E01",
            &["EHZ".to_string()],
            Utc::now() - chrono::Duration::seconds(60),
            Utc::now(),
            "0",
            0.0,
            "http://localhost:8080",
            tmp_dir.path(),
        )
        .await;

        assert!(result.is_none(), "capture_screenshot should return None on connection refused");
    }
}








