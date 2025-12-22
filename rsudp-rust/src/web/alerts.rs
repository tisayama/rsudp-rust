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

use crate::web::plot::draw_rsudp_plot;

use std::collections::HashMap;



pub fn generate_snapshot(



    id: Uuid,



    station: &str,



    channel_data: &HashMap<String, Vec<f64>>,



    start_time: DateTime<Utc>,



    sensitivity: Option<f64>,



    max_intensity: f64,



) -> Result<String, Box<dyn std::error::Error>> {



    let filename = format!("{}.png", id); // No channel in filename if it's a composite



    let path = format!("alerts/{}", filename);







    draw_rsudp_plot(&path, station, channel_data, start_time, 100.0, sensitivity, max_intensity)?;







    Ok(filename)



}








