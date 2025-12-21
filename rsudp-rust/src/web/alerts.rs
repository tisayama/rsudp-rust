use chrono::{DateTime, Utc};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use plotters::prelude::*;
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
) -> Result<(), Box<dyn std::error::Error>> {
    if !settings.email_enabled {
        return Ok(());
    }

    let mut body = format!(
        "Seismic alert RESET on channel {}\nTriggered: {}\nReset: {}\nMax STA/LTA Ratio: {:.2}\n",
        channel,
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

pub fn generate_snapshot(
    id: Uuid,
    channel: &str,
    samples: &[f64],
) -> Result<String, Box<dyn std::error::Error>> {
    let filename = format!("{}_{}.png", id, channel);
    let path = format!("alerts/{}", filename);

    // Matplotlib-like colors
    let bg_color = RGBColor(255, 255, 255);
    let line_color = RGBColor(31, 119, 180); // Matplotlib Tab10 Blue
    let grid_color = RGBColor(220, 220, 220);

    let root = BitMapBackend::new(&path, (1000, 500)).into_drawing_area();
    root.fill(&bg_color)?;

    // Find dynamic scale
    let max_val = samples.iter().fold(0.0f64, |a, &b| a.max(b.abs()));
    let y_limit = if max_val > 0.0 { max_val * 1.1 } else { 1000.0 };

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(0..samples.len(), -y_limit..y_limit)?;

    chart
        .configure_mesh()
        .disable_x_mesh() // Use custom vertical lines if needed, or stick to simple
        .light_line_style(grid_color)
        .x_desc("Samples (100Hz)")
        .y_desc("Amplitude (Counts)")
        .axis_desc_style(("sans-serif", 15))
        .draw()?;

    chart.draw_series(LineSeries::new(
        samples.iter().enumerate().map(|(i, &s)| (i, s)),
        line_color.stroke_width(1),
    ))?;

    root.present()?;

    Ok(filename)
}