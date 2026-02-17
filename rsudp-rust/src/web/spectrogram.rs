use rustfft::{FftPlanner, num_complex::Complex};

pub struct Spectrogram {
    pub frequencies: Vec<f64>,
    pub times: Vec<f64>,
    pub data: Vec<Vec<f64>>, // [time][frequency]
}

pub fn compute_spectrogram(samples: &[f64], sample_rate: f64, nfft: usize, noverlap: usize) -> Spectrogram {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(nfft);

    let step = nfft - noverlap;
    let freq_bins = nfft / 2 + 1;
    let mut data = Vec::new();
    let mut times = Vec::new();

    // Hanning window
    let window: Vec<f64> = (0..nfft)
        .map(|i| 0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / (nfft - 1) as f64).cos()))
        .collect();

    // PSD normalization factor: Fs * sum(window^2)
    let window_power_sum: f64 = window.iter().map(|w| w * w).sum();
    let psd_norm = sample_rate * window_power_sum;

    let time_offset = (nfft as f64 / 2.0) / sample_rate;

    for i in (0..samples.len().saturating_sub(nfft)).step_by(step) {
        let chunk = &samples[i..i + nfft];
        if chunk.len() < nfft { break; }

        let mean = chunk.iter().sum::<f64>() / nfft as f64;

        let mut buffer: Vec<Complex<f64>> = chunk.iter().zip(window.iter())
            .map(|(&s, &w)| Complex { re: (s - mean) * w, im: 0.0 })
            .collect();

        fft.process(&mut buffer);

        // PSD normalization + one-sided correction (linear PSD, no dB)
        let psd_linear: Vec<f64> = buffer.iter().take(freq_bins).enumerate()
            .map(|(k, c)| {
                let mag_sq = c.re * c.re + c.im * c.im;
                let mut psd = mag_sq / psd_norm;
                // One-sided spectrum: double non-DC, non-Nyquist bins
                if k > 0 && k < freq_bins - 1 { psd *= 2.0; }
                psd
            })
            .collect();

        data.push(psd_linear);
        times.push((i as f64 / sample_rate) + time_offset);
    }

    let frequencies: Vec<f64> = (0..freq_bins)
        .map(|i| i as f64 * sample_rate / nfft as f64)
        .collect();

    Spectrogram { frequencies, times, data }
}

#[derive(Debug, Clone)]
pub struct SpectrogramU8 {
    pub frequency_bins: usize,   // NFFT/2 + 1
    pub sample_rate: f64,
    pub columns: Vec<Vec<u8>>,   // [time_column][frequency_bin] -- u8 (0-255)
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

    // Compress all values: PSD^0.1 (matching rsudp's sg ** (1/10))
    let compressed: Vec<Vec<f64>> = spec.data.iter().map(|row| {
        row.iter().map(|&psd| psd.max(0.0).powf(0.1)).collect()
    }).collect();

    // Min-max normalization (matching matplotlib's imshow auto-scaling)
    let mut min_val: f64 = f64::MAX;
    let mut max_val: f64 = f64::MIN;
    for row in &compressed {
        for &val in row {
            if val < min_val { min_val = val; }
            if val > max_val { max_val = val; }
        }
    }
    let range = if max_val > min_val { max_val - min_val } else { 1.0 };

    let columns: Vec<Vec<u8>> = compressed.iter().map(|row| {
        row.iter().map(|&val| {
            let normalized = ((val - min_val) / range).clamp(0.0, 1.0);
            (normalized * 255.0).round() as u8
        }).collect()
    }).collect();

    SpectrogramU8 {
        frequency_bins,
        sample_rate,
        columns,
        timestamps: spec.times,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_spectrogram_u8() {
        let sample_rate = 100.0;
        let nfft = 128;
        let noverlap = (nfft as f64 * 0.9) as usize;
        let duration = 5.0;
        let n_samples = (duration * sample_rate) as usize;

        let freq = 10.0;
        let samples: Vec<f64> = (0..n_samples)
            .map(|i| (2.0 * std::f64::consts::PI * freq * i as f64 / sample_rate).sin() * 1000.0)
            .collect();

        let spec = compute_spectrogram_u8(&samples, sample_rate, nfft, noverlap);

        assert_eq!(spec.frequency_bins, nfft / 2 + 1);
        assert!(!spec.columns.is_empty());
        assert_eq!(spec.columns[0].len(), nfft / 2 + 1);

        for col in &spec.columns {
            assert_eq!(col.len(), nfft / 2 + 1);
        }

        let expected_bin = (freq * nfft as f64 / sample_rate).round() as usize;
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
