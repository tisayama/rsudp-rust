use rustfft::{FftPlanner, num_complex::Complex};
use plotters::prelude::*;
use plotters::style::text_anchor::{Pos, HPos, VPos};
use chrono::{DateTime, Utc, TimeZone};
use std::collections::HashMap;
use crate::intensity::get_shindo_class;
use tracing::info;

pub struct Spectrogram {
    pub frequencies: Vec<f64>,
    pub times: Vec<f64>,
    pub data: Vec<Vec<f64>>, // [time][frequency]
}

pub fn compute_spectrogram(samples: &[f64], sample_rate: f64, nfft: usize, noverlap: usize) -> Spectrogram {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(nfft);
    
    let step = nfft - noverlap;
    let mut data = Vec::new();
    let mut times = Vec::new();
    
    // Hanning window
    let window: Vec<f64> = (0..nfft)
        .map(|i| 0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / (nfft - 1) as f64).cos()))
        .collect();

    let time_offset = (nfft as f64 / 2.0) / sample_rate;

    for i in (0..samples.len().saturating_sub(nfft)).step_by(step) {
        let chunk = &samples[i..i + nfft];
        if chunk.len() < nfft { break; }
        
        let mean = chunk.iter().sum::<f64>() / nfft as f64;
        
        let mut buffer: Vec<Complex<f64>> = chunk.iter().zip(window.iter())
            .map(|(&s, &w)| Complex { re: (s - mean) * w, im: 0.0 })
            .collect();
            
        fft.process(&mut buffer);
        
        let psd: Vec<f64> = buffer.iter().take(nfft / 2 + 1)
            .map(|c| {
                let mag_sq = c.re * c.re + c.im * c.im;
                mag_sq
            })
            .collect();
            
        data.push(psd);
        times.push((i as f64 / sample_rate) + time_offset);
    }
    
    let frequencies: Vec<f64> = (0..nfft / 2 + 1)
        .map(|i| i as f64 * sample_rate / nfft as f64)
        .collect();
        
    Spectrogram { frequencies, times, data }
}

#[derive(Debug, Clone)]
pub struct SpectrogramU8 {
    pub frequency_bins: usize,   // NFFT/2 + 1
    pub sample_rate: f64,
    pub columns: Vec<Vec<u8>>,   // [time_column][frequency_bin] — u8 (0-255)
    pub timestamps: Vec<f64>,    // each column's relative time (seconds)
}

pub fn compute_spectrogram_u8(samples: &[f64], sample_rate: f64, nfft: usize, noverlap: usize) -> SpectrogramU8 {
    let spec = compute_spectrogram(samples, sample_rate, nfft, noverlap);

    let frequency_bins = nfft / 2 + 1;

    if spec.data.is_empty() {
        return SpectrogramU8 {
            frequency_bins,
            sample_rate,
            columns: Vec::new(),
            timestamps: spec.times,
        };
    }

    // Find global max for auto-normalization
    let mut max_mag_sq: f64 = 1e-10;
    for row in &spec.data {
        for &val in row {
            if val > max_mag_sq {
                max_mag_sq = val;
            }
        }
    }

    // Apply power scaling (^1/10) and normalize to 0-255
    let columns: Vec<Vec<u8>> = spec.data.iter().map(|row| {
        row.iter().map(|&mag_sq| {
            let normalized = (mag_sq / max_mag_sq).powf(0.1);
            (normalized * 255.0).round().min(255.0).max(0.0) as u8
        }).collect()
    }).collect();

    SpectrogramU8 {
        frequency_bins,
        sample_rate,
        columns,
        timestamps: spec.times,
    }
}

fn get_jma_color(shindo: &str) -> RGBColor {
    match shindo {
        "0" => RGBColor(242, 242, 255),
        "1" => RGBColor(160, 238, 255),
        "2" => RGBColor(0, 187, 255),
        "3" => RGBColor(51, 255, 0),
        "4" => RGBColor(255, 255, 0),
        "5-" => RGBColor(255, 153, 0),
        "5+" => RGBColor(255, 40, 0),
        "6-" => RGBColor(165, 0, 33),
        "6+" => RGBColor(85, 0, 17),
        "7" => RGBColor(85, 0, 85),
        _ => RGBColor(255, 255, 255),
    }
}

pub fn draw_rsudp_plot(
    path: &str,
    station: &str,
    channel_data: &HashMap<String, Vec<f64>>,
    start_time: DateTime<Utc>,
    sample_rate: f64,
    sensitivity: Option<f64>,
    max_intensity: f64, // Explicitly passed from pipeline
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Drawing plot for {}. Start Time: {}, Sample Rate: {}", station, start_time, sample_rate);
    
    // Bake the fonts into the binary for static lifetime
    const DEJAVU_FONT: &[u8] = include_bytes!("../resources/DejaVuSansCondensed.ttf");
    const NOTO_FONT: &[u8] = include_bytes!("../resources/NotoSansJP-Bold.otf");
    
    let _ = plotters::style::register_font("sans-serif", FontStyle::Normal, DEJAVU_FONT);
    let _ = plotters::style::register_font("noto-sans", FontStyle::Normal, NOTO_FONT);

    let n_channels = channel_data.len();
    let width = 1000;
    let height = 500 * n_channels as u32;

    let shindo = get_shindo_class(max_intensity);
    let shindo_color = get_jma_color(&shindo);

    let display_shindo = match shindo.as_str() {
        "5-" => "5弱",
        "5+" => "5強",
        "6-" => "6弱",
        "6+" => "6強",
        _ => &shindo,
    };
    let label = format!("[震度 {}相当]", display_shindo);

    // rsudp Dark Theme Colors
    let bg_color = RGBColor(32, 37, 48); // #202530
    let fg_color = RGBColor(204, 204, 204); // 0.8 grey
    let line_color = RGBColor(194, 130, 133); // #c28285 pinkish

    let root = BitMapBackend::new(path, (width, height)).into_drawing_area();
    root.fill(&bg_color)?;

    let label_font = FontDesc::new(FontFamily::Name("sans-serif"), 12.0, FontStyle::Normal).color(&fg_color);
    let title_font = FontDesc::new(FontFamily::Name("sans-serif"), 18.0, FontStyle::Normal).color(&fg_color);

    let colormap_lut: Vec<RGBColor> = (0..256)
        .map(|i| {
            let color = colorous::INFERNO.eval_rational(i, 255);
            RGBColor(color.r, color.g, color.b)
        })
        .collect();

    let channel_areas = root.split_evenly((n_channels, 1));

    let mut sorted_channels: Vec<_> = channel_data.keys().collect();
    sorted_channels.sort_by(|a, b| {
        let order = |s: &str| {
            if s.ends_with('Z') { 0 }
            else if s.ends_with('N') { 1 }
            else if s.ends_with('E') { 2 }
            else { 3 }
        };
        order(a).cmp(&order(b)).then(a.cmp(b))
    });

    for (i, &chan) in sorted_channels.iter().enumerate() {
        let raw_samples = &channel_data[chan];
        
        let mean_val = raw_samples.iter().sum::<f64>() / raw_samples.len() as f64;
        let mut samples: Vec<f64> = raw_samples.iter().map(|&s| s - mean_val).collect();

        let unit_label = if let Some(sens_val) = sensitivity {
            if sens_val > 0.0 {
                for s in &mut samples {
                    *s /= sens_val;
                }
                if sens_val > 1_000_000.0 { "Velocity (m/s)" } else { "Accel (m/s²)" }
            } else {
                "Counts"
            }
        } else {
            "Counts"
        };

        let total_seconds = samples.len() as f64 / sample_rate;
        let start_ts = start_time.timestamp_millis();
        let end_ts = start_ts + (total_seconds * 1000.0) as i64;
        
        let channel_area = &channel_areas[i];
        let (waveform_area, spectrogram_area) = channel_area.split_vertically(250);

        let pga = samples.iter().fold(0.0f64, |a, &b| a.max(b.abs()));

        // 1. Waveform
        let y_limit = if pga > 0.0 { pga * 1.1 } else { 1000.0 };

        let mut chart = ChartBuilder::on(&waveform_area)
            .caption(
                format!("{} - {} | Peak: {:.2e} | Start: {}", station, chan, pga, start_time.format("%Y-%m-%d %H:%M:%S UTC")), 
                title_font.clone()
            )
            .margin_left(20)
            .margin_right(20)
            .margin_top(10)
            .set_label_area_size(LabelAreaPosition::Left, 70)
            .build_cartesian_2d(start_ts..end_ts, -y_limit..y_limit)?;

        chart
            .configure_mesh()
            .disable_mesh() // T006: Disable grid for Waveform
            .disable_x_axis() // T005: Hide X-axis for Waveform
            .y_desc(unit_label)
            .axis_desc_style(label_font.clone())
            .label_style(label_font.clone())
            .draw()?;

        chart.draw_series(LineSeries::new(
            samples.iter().enumerate().map(|(i, &s)| {
                let t = start_ts + ((i as f64 / sample_rate) * 1000.0) as i64;
                (t, s)
            }),
            line_color.stroke_width(1),
        ))?;

        // 2. Spectrogram
        let nfft = 128; 
        let overlap = (nfft as f64 * 0.9) as usize; 
        let spec = compute_spectrogram(&samples, sample_rate, nfft, overlap);
        
        if !spec.data.is_empty() {
            let mut max_mag_sq = 1e-10;
            for row in &spec.data {
                for &val in row {
                    if val > max_mag_sq { max_mag_sq = val; }
                }
            }

            let mut spec_chart = ChartBuilder::on(&spectrogram_area)
                .margin_left(20)
                .margin_right(20)
                .margin_bottom(40)
                .set_label_area_size(LabelAreaPosition::Left, 70)
                .set_label_area_size(LabelAreaPosition::Bottom, 30)
                .build_cartesian_2d(start_ts..end_ts, 0.0..sample_rate / 2.0)?;

            spec_chart
                .configure_mesh()
                .disable_mesh()
                .axis_style(ShapeStyle::from(&fg_color).stroke_width(1))
                .x_desc("Time (UTC)")
                .y_desc("Freq [Hz]")
                .axis_desc_style(label_font.clone())
                .label_style(label_font.clone())
                .x_labels(6)
                .x_label_formatter(&|ms| {
                    if let Some(dt) = Utc.timestamp_millis_opt(*ms).single() {
                        dt.format("%H:%M:%S").to_string()
                    } else {
                        "".to_string()
                    }
                })
                .draw()?;

            let x_step_ms = if spec.times.len() > 1 { ((spec.times[1] - spec.times[0]) * 1000.0) as i64 } else { 1000 };
            let y_step = spec.frequencies[1] - spec.frequencies[0];

            for (t_idx, t_sec) in spec.times.iter().enumerate() {
                let t_center = start_ts + (t_sec * 1000.0) as i64;
                let t_start = t_center - x_step_ms / 2;
                let t_end = t_center + x_step_ms / 2;

                for (f_idx, f) in spec.frequencies.iter().enumerate() {
                    let mag_sq = spec.data[t_idx][f_idx];
                    let intensity = (mag_sq / max_mag_sq).powf(0.1);
                    let lut_idx = (intensity * 255.0) as usize;
                    let plot_color = colormap_lut[lut_idx.min(255)];

                    spec_chart.draw_series(std::iter::once(Rectangle::new(
                        [(t_start, *f), (t_end + 20, *f + y_step)], // Add 20ms overlap to prevent visual gaps
                        plot_color.filled(),
                    )))?;
                }
            }
        }
    }

    // 3. Draw Intensity Badge (Top-Right)
    let badge_width = 400; // Increased for longer text
    let badge_height = 100;
    let badge_margin = 30;
    let badge_rect = [
        (width as i32 - badge_width - badge_margin, badge_margin), 
        (width as i32 - badge_margin, badge_height + badge_margin)
    ];
    
    root.draw(&Rectangle::new(badge_rect, shindo_color.filled()))?;
    
    let badge_font = FontDesc::new(FontFamily::Name("noto-sans"), 40.0, FontStyle::Normal).color(&WHITE);
    let badge_text_style = TextStyle::from(badge_font)
        .pos(Pos::new(HPos::Center, VPos::Center));
    
    root.draw(&Text::new(label, (width as i32 - badge_width / 2 - badge_margin, badge_margin + badge_height / 2), badge_text_style))?;

    root.present()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_spectrogram_u8() {
        let sample_rate = 100.0;
        let nfft = 128;
        let noverlap = (nfft as f64 * 0.9) as usize; // 90% overlap
        let duration = 5.0;
        let n_samples = (duration * sample_rate) as usize;

        // Generate 10 Hz sine wave
        let freq = 10.0;
        let samples: Vec<f64> = (0..n_samples)
            .map(|i| (2.0 * std::f64::consts::PI * freq * i as f64 / sample_rate).sin() * 1000.0)
            .collect();

        let spec = compute_spectrogram_u8(&samples, sample_rate, nfft, noverlap);

        assert_eq!(spec.frequency_bins, nfft / 2 + 1);
        assert!(!spec.columns.is_empty());
        assert_eq!(spec.columns[0].len(), nfft / 2 + 1);

        // Check all columns have correct length
        for col in &spec.columns {
            assert_eq!(col.len(), nfft / 2 + 1);
        }

        // The peak should be at the 10 Hz frequency bin
        // bin index = freq * nfft / sample_rate = 10 * 128 / 100 = 12.8 ≈ 13
        let expected_bin = (freq * nfft as f64 / sample_rate).round() as usize;
        // Check that the expected bin has a high value (close to 255) in at least one column
        let max_at_peak = spec.columns.iter()
            .map(|col| col[expected_bin])
            .max()
            .unwrap_or(0);
        assert!(max_at_peak > 200, "Expected peak bin {} to have high value, got {}", expected_bin, max_at_peak);
    }

    #[test]
    fn test_compute_spectrogram_u8_empty() {
        let spec = compute_spectrogram_u8(&[], 100.0, 128, 115);
        assert!(spec.columns.is_empty());
        assert_eq!(spec.frequency_bins, 65);
    }

    #[test]
    fn test_spectrogram_dimensions() {
        let sample_rate = 100.0;
        let nfft = 256;
        let noverlap = 128;
        let seconds = 10.0;
        let samples: Vec<f64> = (0..(seconds * sample_rate) as usize).map(|i| (i as f64).sin()).collect();
        
        let spec = compute_spectrogram(&samples, sample_rate, nfft, noverlap);
        
        assert_eq!(spec.frequencies.len(), nfft / 2 + 1);
        assert!(!spec.data.is_empty());
        assert_eq!(spec.data[0].len(), nfft / 2 + 1);
    }
}