use crate::trigger::Biquad;
use crate::parser::stationxml::ChannelResponse;
use rustfft::{FftPlanner, num_complex::Complex};
use std::f64::consts::PI;

/// Compute Butterworth bandpass filter as cascaded second-order sections (SOS)
/// using the bilinear transform method.
///
/// - `order`: filter order (number of poles per lowpass/highpass, total bandpass order = 2*order)
/// - `low_freq`: lower cutoff frequency (Hz)
/// - `high_freq`: upper cutoff frequency (Hz)
/// - `fs`: sampling frequency (Hz)
pub fn butter_bandpass_sos(order: usize, low_freq: f64, high_freq: f64, fs: f64) -> Vec<Biquad> {
    assert!(order > 0, "Filter order must be > 0");
    assert!(low_freq > 0.0 && low_freq < high_freq, "Invalid frequency range");
    assert!(high_freq < fs / 2.0, "Upper frequency must be below Nyquist");

    // Step 1: Pre-warp digital frequencies to analog domain
    let omega_low = 2.0 * fs * (PI * low_freq / fs).tan();
    let omega_high = 2.0 * fs * (PI * high_freq / fs).tan();
    let omega_0 = (omega_low * omega_high).sqrt(); // center frequency
    let bw = omega_high - omega_low; // bandwidth

    // Step 2: Analog Butterworth lowpass prototype poles (left half-plane only)
    // For order N, poles are at: s_k = exp(j * pi * (2k + N + 1) / (2N))
    // Poles come in conjugate pairs: k and (N-1-k). We only process unique poles
    // (the upper half-plane ones) to avoid duplicating biquad sections.
    // For even N: N/2 unique pairs. For odd N: (N-1)/2 pairs + 1 real pole.
    let mut sections = Vec::new();

    let n_unique = (order + 1) / 2; // ceil(order/2)
    for k in 0..n_unique {
        let theta = PI * (2 * k + order + 1) as f64 / (2 * order) as f64;
        let p_re = theta.cos();
        let p_im = theta.sin();

        // Step 3: Lowpass-to-bandpass transform
        // Each 1st-order lowpass pole p becomes two bandpass poles:
        //   s = (p * bw ± sqrt((p*bw)^2 - 4*omega_0^2)) / 2
        let pbw_re = p_re * bw;
        let pbw_im = p_im * bw;

        // (p*bw)^2 as complex number
        let pbw2_re = pbw_re * pbw_re - pbw_im * pbw_im;
        let pbw2_im = 2.0 * pbw_re * pbw_im;

        // discriminant = (p*bw)^2 - 4*omega_0^2
        let disc_re = pbw2_re - 4.0 * omega_0 * omega_0;
        let disc_im = pbw2_im;

        // Complex square root of discriminant
        let disc_mag = (disc_re * disc_re + disc_im * disc_im).sqrt();
        let disc_arg = disc_im.atan2(disc_re);
        let sqrt_disc_mag = disc_mag.sqrt();
        let sqrt_disc_re = sqrt_disc_mag * (disc_arg / 2.0).cos();
        let sqrt_disc_im = sqrt_disc_mag * (disc_arg / 2.0).sin();

        // Two analog bandpass poles: s = (p*bw ± sqrt(disc)) / 2
        let s1_re = (pbw_re + sqrt_disc_re) / 2.0;
        let s1_im = (pbw_im + sqrt_disc_im) / 2.0;
        let s2_re = (pbw_re - sqrt_disc_re) / 2.0;
        let s2_im = (pbw_im - sqrt_disc_im) / 2.0;

        // Step 4: Bilinear transform each pole pair into a digital biquad section
        // Each unique prototype pole generates two biquad sections (one per bandpass pole pair).

        // Section for pole s1 (and its conjugate s1*)
        let bq1 = analog_pole_pair_to_biquad(s1_re, s1_im, omega_0, fs);
        sections.push(bq1);

        // Section for pole s2 (and its conjugate s2*)
        let bq2 = analog_pole_pair_to_biquad(s2_re, s2_im, omega_0, fs);
        sections.push(bq2);
    }

    // Normalize gain: compute and apply gain correction at center frequency
    let w0 = 2.0 * PI * (low_freq * high_freq).sqrt() / fs;
    let mut gain_re = 1.0;
    let mut gain_im = 0.0;
    let ej_re = w0.cos();
    let ej_im = w0.sin();

    for section in &sections {
        // Numerator: b0 + b1*z^-1 + b2*z^-2  evaluated at z=e^(jw)
        let num_re = section.b0 + section.b1 * ej_re + section.b2 * (2.0 * ej_re * ej_re - 1.0);
        let num_im = -section.b1 * ej_im - section.b2 * 2.0 * ej_re * ej_im;
        // Denominator: 1 + a1*z^-1 + a2*z^-2
        let den_re = 1.0 + section.a1 * ej_re + section.a2 * (2.0 * ej_re * ej_re - 1.0);
        let den_im = -section.a1 * ej_im - section.a2 * 2.0 * ej_re * ej_im;

        // H(z) = num / den (complex division)
        let den_mag2 = den_re * den_re + den_im * den_im;
        let h_re = (num_re * den_re + num_im * den_im) / den_mag2;
        let h_im = (num_im * den_re - num_re * den_im) / den_mag2;

        // Multiply into cumulative gain
        let new_re = gain_re * h_re - gain_im * h_im;
        let new_im = gain_re * h_im + gain_im * h_re;
        gain_re = new_re;
        gain_im = new_im;
    }

    let total_gain = (gain_re * gain_re + gain_im * gain_im).sqrt();
    if total_gain > 1e-15 {
        // Distribute gain correction across all sections evenly
        let per_section_gain = total_gain.powf(1.0 / sections.len() as f64);
        for section in &mut sections {
            section.b0 /= per_section_gain;
            section.b1 /= per_section_gain;
            section.b2 /= per_section_gain;
        }
    }

    sections
}

/// Convert an analog pole pair (pole and its conjugate) to a digital biquad section
/// for a bandpass filter using bilinear transform.
fn analog_pole_pair_to_biquad(p_re: f64, p_im: f64, _omega_0: f64, fs: f64) -> Biquad {
    let t = 1.0 / (2.0 * fs);

    // Bilinear transform: z-domain pole from s-domain pole
    // z = (1 + s*T) / (1 - s*T) where T = 1/(2*fs)
    // For a conjugate pair at p ± j*p_im, the 2nd-order section denominator is:
    // (z - z1)(z - z1*) = z^2 - 2*Re(z1)*z + |z1|^2

    let num_re = 1.0 + p_re * t;
    let num_im = p_im * t;
    let den_re = 1.0 - p_re * t;
    let den_im = -p_im * t;

    let den_mag2 = den_re * den_re + den_im * den_im;
    let z_re = (num_re * den_re + num_im * den_im) / den_mag2;
    let z_im = (num_im * den_re - num_re * den_im) / den_mag2;

    // Denominator coefficients: 1 + a1*z^-1 + a2*z^-2
    let a1 = -2.0 * z_re;
    let a2 = z_re * z_re + z_im * z_im;

    // Numerator for bandpass: (z^2 - 1) / normalizing factor
    // Bandpass zero at z=1 and z=-1 → numerator is (1 - z^-2) = z^-0 + 0*z^-1 - 1*z^-2
    let b0 = 1.0;
    let b1 = 0.0;
    let b2 = -1.0;

    Biquad::new(b0, b1, b2, a1, a2)
}

/// A chain of Biquad sections with persistent state for streaming filtering.
#[derive(Debug, Clone)]
pub struct BiquadChain {
    sections: Vec<Biquad>,
}

impl BiquadChain {
    pub fn new(sections: Vec<Biquad>) -> Self {
        Self { sections }
    }

    /// Create a bandpass filter chain.
    pub fn bandpass(order: usize, low_freq: f64, high_freq: f64, fs: f64) -> Self {
        Self::new(butter_bandpass_sos(order, low_freq, high_freq, fs))
    }

    /// Process a single sample through the cascade.
    pub fn process(&mut self, x: f64) -> f64 {
        let mut val = x;
        for section in &mut self.sections {
            val = section.process(val);
        }
        val
    }

    /// Process a slice of samples, returning filtered output.
    pub fn process_vec(&mut self, samples: &[f64]) -> Vec<f64> {
        samples.iter().map(|&s| self.process(s)).collect()
    }

    /// Zero-phase (forward-backward) filtering, equivalent to scipy's sosfiltfilt.
    /// Uses odd-symmetric signal extension to minimize edge transients,
    /// matching scipy's default `padtype='odd'` behavior.
    /// Note: this resets filter state and is meant for batch/offline processing.
    pub fn filtfilt(&mut self, samples: &[f64]) -> Vec<f64> {
        if samples.is_empty() {
            return Vec::new();
        }
        let n = samples.len();
        if n < 2 {
            return samples.to_vec();
        }

        // Pad length: 3 × number of biquad sections (matching scipy's default)
        let pad_len = (3 * self.sections.len()).min(n - 1);

        // Build extended signal with odd-symmetric reflections:
        // [2*x[0]-x[pad_len], ..., 2*x[0]-x[1], x[0], x[1], ..., x[n-1], 2*x[n-1]-x[n-2], ..., 2*x[n-1]-x[n-pad_len]]
        let mut extended = Vec::with_capacity(n + 2 * pad_len);

        // Left extension (odd reflection around samples[0])
        for i in (1..=pad_len).rev() {
            extended.push(2.0 * samples[0] - samples[i]);
        }
        // Original signal
        extended.extend_from_slice(samples);
        // Right extension (odd reflection around samples[n-1])
        for i in 1..=pad_len {
            extended.push(2.0 * samples[n - 1] - samples[n - 1 - i]);
        }

        // Forward pass
        self.reset();
        let mut forward: Vec<f64> = extended.iter().map(|&s| self.process(s)).collect();

        // Reverse
        forward.reverse();

        // Backward pass
        self.reset();
        let mut backward: Vec<f64> = forward.iter().map(|&s| self.process(s)).collect();

        // Reverse to restore order
        backward.reverse();

        // Trim padding
        backward[pad_len..pad_len + n].to_vec()
    }

    /// Reset filter state (zero all delay elements).
    pub fn reset(&mut self) {
        for section in &mut self.sections {
            section.s1 = 0.0;
            section.s2 = 0.0;
        }
    }
}

/// Evaluate the instrument response at a given angular frequency (rad/s).
/// Returns a complex value: H(jw) = A0 * product(jw - z_i) / product(jw - p_j)
fn evaluate_response_at(response: &ChannelResponse, omega: f64) -> Complex<f64> {
    let jw = Complex::new(0.0, omega);

    let mut numerator = Complex::new(response.normalization_factor, 0.0);
    for &(re, im) in &response.zeros {
        numerator *= jw - Complex::new(re, im);
    }

    let mut denominator = Complex::new(1.0, 0.0);
    for &(re, im) in &response.poles {
        denominator *= jw - Complex::new(re, im);
    }

    if denominator.norm() < 1e-30 {
        return Complex::new(0.0, 0.0);
    }

    (numerator / denominator) * response.stage_gain
}

/// Compute the cosine taper weight for the pre-filter at a given frequency.
/// pre_filt = [f1, f2, f3, f4]:
///   - below f1: 0.0
///   - f1 to f2: cosine taper 0 → 1
///   - f2 to f3: 1.0 (passband)
///   - f3 to f4: cosine taper 1 → 0
///   - above f4: 0.0
fn pre_filter_weight(freq: f64, pre_filt: &[f64; 4]) -> f64 {
    if freq <= pre_filt[0] || freq >= pre_filt[3] {
        0.0
    } else if freq < pre_filt[1] {
        let t = (freq - pre_filt[0]) / (pre_filt[1] - pre_filt[0]);
        0.5 * (1.0 - (PI * t).cos())
    } else if freq <= pre_filt[2] {
        1.0
    } else {
        let t = (freq - pre_filt[2]) / (pre_filt[3] - pre_filt[2]);
        0.5 * (1.0 + (PI * t).cos())
    }
}

/// Apply frequency-domain deconvolution to a batch of samples.
/// This matches obspy's `Trace.remove_response()` with output='VEL'.
///
/// - `samples`: raw ADC counts (f64)
/// - `response`: instrument response (poles/zeros)
/// - `sample_rate`: samples per second
/// - `pre_filt`: [f1, f2, f3, f4] Hz — cosine taper to avoid low/high frequency blowup
/// - `water_level_db`: minimum response level in dB (relative to max) to prevent division by near-zero
///
/// Returns deconvolved samples in physical units (m/s for velocity instruments).
pub fn deconvolve_response(
    samples: &[f64],
    response: &ChannelResponse,
    sample_rate: f64,
    pre_filt: [f64; 4],
    water_level_db: f64,
) -> Vec<f64> {
    let n = samples.len();
    if n == 0 || response.poles.is_empty() {
        // No response data — fall back to simple sensitivity division
        if response.sensitivity > 0.0 {
            return samples.iter().map(|&s| s / response.sensitivity).collect();
        }
        return samples.to_vec();
    }

    // Pad to at least 2*n to prevent circular convolution wrap-around,
    // matching obspy's _npts2nfft(). Then round to next power of 2 for FFT efficiency.
    let nfft = (2 * n).next_power_of_two().max(512);

    // Prepare FFT input: mean-removed signal + zero-padding
    let mean = samples.iter().sum::<f64>() / n as f64;
    let mut fft_input: Vec<Complex<f64>> = samples.iter()
        .map(|&s| Complex::new(s - mean, 0.0))
        .collect();
    fft_input.resize(nfft, Complex::new(0.0, 0.0));

    // Forward FFT
    let mut planner = FftPlanner::new();
    let fft_forward = planner.plan_fft_forward(nfft);
    fft_forward.process(&mut fft_input);

    // Compute response at each frequency bin
    let freq_resolution = sample_rate / nfft as f64;
    let mut response_values: Vec<Complex<f64>> = Vec::with_capacity(nfft);
    let mut max_response_mag = 0.0_f64;

    for k in 0..nfft {
        let freq = if k <= nfft / 2 {
            k as f64 * freq_resolution
        } else {
            (k as f64 - nfft as f64) * freq_resolution
        };
        let omega = 2.0 * PI * freq;
        let r = evaluate_response_at(response, omega);
        max_response_mag = max_response_mag.max(r.norm());
        response_values.push(r);
    }

    // Apply water level: minimum response magnitude in linear scale
    let water_level_linear = max_response_mag * 10.0_f64.powf(-water_level_db / 20.0);

    // Deconvolve in frequency domain
    for k in 0..nfft {
        let freq = if k <= nfft / 2 {
            k as f64 * freq_resolution
        } else {
            (k as f64 - nfft as f64) * freq_resolution
        };
        let abs_freq = freq.abs();

        // Pre-filter weight
        let weight = pre_filter_weight(abs_freq, &pre_filt);

        if weight < 1e-10 {
            fft_input[k] = Complex::new(0.0, 0.0);
            continue;
        }

        let r = response_values[k];
        let r_mag = r.norm();

        // Apply water level: if response is too small, clip to water level
        let effective_r = if r_mag < water_level_linear {
            // Preserve phase, use water_level magnitude
            if r_mag > 1e-30 {
                r * (water_level_linear / r_mag)
            } else {
                Complex::new(water_level_linear, 0.0)
            }
        } else {
            r
        };

        // Deconvolve: divide by response, apply pre-filter
        fft_input[k] = fft_input[k] * weight / effective_r;
    }

    // Inverse FFT
    let fft_inverse = planner.plan_fft_inverse(nfft);
    fft_inverse.process(&mut fft_input);

    // Normalize (rustfft doesn't normalize) and extract real part
    let scale = 1.0 / nfft as f64;
    fft_input[..n].iter().map(|c| c.re * scale).collect()
}

/// Streaming deconvolution state for real-time processing.
/// Uses overlap-save method to process incoming data in blocks.
pub struct StreamingDeconvolver {
    response: ChannelResponse,
    sample_rate: f64,
    pre_filt: [f64; 4],
    water_level_db: f64,
    /// Accumulated input buffer
    buffer: Vec<f64>,
    /// Overlap from previous block (for continuity)
    overlap: Vec<f64>,
    /// Minimum block size for FFT processing
    block_size: usize,
    /// Overlap size (for smooth transitions between blocks)
    overlap_size: usize,
}

impl StreamingDeconvolver {
    pub fn new(
        response: ChannelResponse,
        sample_rate: f64,
        pre_filt: [f64; 4],
        water_level_db: f64,
    ) -> Self {
        let block_size = 256; // ~2.56 seconds at 100 Hz
        let overlap_size = 128; // ~1.28 seconds overlap
        Self {
            response,
            sample_rate,
            pre_filt,
            water_level_db,
            buffer: Vec::new(),
            overlap: Vec::new(),
            block_size,
            overlap_size,
        }
    }

    /// Add new samples and return any fully deconvolved output.
    /// Returns (deconvolved_samples, timestamp_offset_samples) where
    /// timestamp_offset_samples is how many samples from the START of the
    /// accumulated input correspond to the beginning of the output.
    pub fn process(&mut self, samples: &[f64]) -> Vec<f64> {
        self.buffer.extend_from_slice(samples);

        let mut output = Vec::new();

        while self.buffer.len() >= self.block_size {
            // Build processing segment: overlap + new block
            let mut segment = Vec::with_capacity(self.overlap_size + self.block_size);
            segment.extend_from_slice(&self.overlap);
            segment.extend_from_slice(&self.buffer[..self.block_size]);

            // Deconvolve the full segment
            let deconv = deconvolve_response(
                &segment,
                &self.response,
                self.sample_rate,
                self.pre_filt,
                self.water_level_db,
            );

            // Keep only the new part (discard overlap region from output)
            let valid_start = self.overlap.len();
            if deconv.len() > valid_start {
                output.extend_from_slice(&deconv[valid_start..]);
            }

            // Save overlap for next block
            let new_overlap_start = self.block_size.saturating_sub(self.overlap_size);
            self.overlap = self.buffer[new_overlap_start..self.block_size].to_vec();

            // Remove processed samples from buffer
            self.buffer.drain(..self.block_size);
        }

        output
    }

    /// Flush remaining buffered samples (for end of stream or settings change).
    pub fn flush(&mut self) -> Vec<f64> {
        if self.buffer.is_empty() {
            return Vec::new();
        }

        let mut segment = Vec::with_capacity(self.overlap.len() + self.buffer.len());
        segment.extend_from_slice(&self.overlap);
        segment.extend_from_slice(&self.buffer);

        let deconv = deconvolve_response(
            &segment,
            &self.response,
            self.sample_rate,
            self.pre_filt,
            self.water_level_db,
        );

        let valid_start = self.overlap.len();
        self.buffer.clear();
        self.overlap.clear();

        if deconv.len() > valid_start {
            deconv[valid_start..].to_vec()
        } else {
            Vec::new()
        }
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.overlap.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bandpass_basic_properties() {
        // Order 4, 0.7-2.0 Hz, fs=100
        let sections = butter_bandpass_sos(4, 0.7, 2.0, 100.0);
        // Order 4 bandpass: ceil(4/2) * 2 = 4 biquad sections
        assert_eq!(sections.len(), 4);

        // All poles should be inside unit circle (stable filter)
        for s in &sections {
            // For stability: |a2| < 1
            assert!(s.a2.abs() < 1.0 + 1e-10, "Unstable section: a2={}", s.a2);
        }
    }

    #[test]
    fn test_bandpass_passband_gain() {
        // The filter should pass signals at the center frequency
        let mut chain = BiquadChain::bandpass(4, 0.7, 2.0, 100.0);

        // Generate a sine wave at center frequency (~1.18 Hz)
        let center_freq = (0.7_f64 * 2.0).sqrt();
        let fs = 100.0;
        let n = 2000; // 20 seconds of data

        // Warm up the filter with a few seconds first
        let warmup: Vec<f64> = (0..500)
            .map(|i| (2.0 * PI * center_freq * i as f64 / fs).sin())
            .collect();
        for &s in &warmup {
            chain.process(s);
        }

        // Measure output amplitude in steady state
        let mut max_output = 0.0_f64;
        for i in 0..n {
            let input = (2.0 * PI * center_freq * (i + 500) as f64 / fs).sin();
            let output = chain.process(input);
            max_output = max_output.max(output.abs());
        }

        // At center frequency, gain should be close to 1.0 (within 3dB = factor of 0.7)
        assert!(max_output > 0.5, "Passband gain too low: {}", max_output);
        assert!(max_output < 2.0, "Passband gain too high: {}", max_output);
    }

    #[test]
    fn test_bandpass_stopband_rejection() {
        let mut chain = BiquadChain::bandpass(4, 0.7, 2.0, 100.0);

        // Test rejection at 10 Hz (well outside passband)
        let test_freq = 10.0;
        let fs = 100.0;

        // Warm up
        for i in 0..1000 {
            let input = (2.0 * PI * test_freq * i as f64 / fs).sin();
            chain.process(input);
        }

        // Measure output
        let mut max_output = 0.0_f64;
        for i in 1000..2000 {
            let input = (2.0 * PI * test_freq * i as f64 / fs).sin();
            let output = chain.process(input);
            max_output = max_output.max(output.abs());
        }

        // At 10 Hz, should be strongly attenuated (order 4 bandpass = steep rolloff)
        assert!(max_output < 0.1, "Stopband rejection insufficient: {}", max_output);
    }

    #[test]
    fn test_filtfilt_passband_unity_gain() {
        // Zero-phase filter should have gain very close to 1.0 at center frequency
        let mut chain = BiquadChain::bandpass(4, 0.7, 2.0, 100.0);
        let center_freq = (0.7_f64 * 2.0).sqrt();
        let fs = 100.0;

        // Generate 10 seconds of sine at center frequency
        let n = 1000;
        let signal: Vec<f64> = (0..n)
            .map(|i| (2.0 * PI * center_freq * i as f64 / fs).sin())
            .collect();

        let filtered = chain.filtfilt(&signal);

        // Measure amplitude in the middle (avoid edge effects)
        let mid_start = n / 4;
        let mid_end = 3 * n / 4;
        let max_output = filtered[mid_start..mid_end].iter()
            .map(|&x| x.abs())
            .fold(0.0_f64, f64::max);

        // filtfilt should give gain very close to 1.0 (within 15%)
        // The doubled effective order from forward-backward pass preserves passband amplitude
        // much better than forward-only filtering (~0.7 gain)
        assert!(max_output > 0.85, "filtfilt passband gain too low: {}", max_output);
        assert!(max_output < 1.15, "filtfilt passband gain too high: {}", max_output);
    }

    #[test]
    fn test_biquad_chain_reset() {
        let mut chain = BiquadChain::bandpass(2, 1.0, 5.0, 100.0);

        // Process some data
        for i in 0..100 {
            chain.process(i as f64);
        }

        // Reset
        chain.reset();

        // Verify all states are zero
        for section in &chain.sections {
            assert_eq!(section.s1, 0.0);
            assert_eq!(section.s2, 0.0);
        }
    }

    #[test]
    fn test_evaluate_response_rs4d_ehz() {
        // RS4D EHZ known response from FDSN
        let response = ChannelResponse {
            zeros: vec![(0.0, 0.0), (0.0, 0.0), (0.0, 0.0)],
            poles: vec![(-1.0, 0.0), (-3.03, 0.0), (-3.03, 0.0), (-666.67, 0.0)],
            normalization_factor: 673.744,
            stage_gain: 399_650_000.0,
            sensitivity: 399_650_000.0,
        };

        // At reference frequency (5 Hz), |H| should ≈ sensitivity
        let omega_5hz = 2.0 * PI * 5.0;
        let h_5hz = evaluate_response_at(&response, omega_5hz);
        let ratio_5hz = h_5hz.norm() / response.sensitivity;
        assert!(
            (ratio_5hz - 1.0).abs() < 0.01,
            "Response at 5 Hz should be ~sensitivity, got ratio={}",
            ratio_5hz
        );

        // At 0.5 Hz, response should be ~50% of sensitivity
        let omega_05hz = 2.0 * PI * 0.5;
        let h_05hz = evaluate_response_at(&response, omega_05hz);
        let ratio_05hz = h_05hz.norm() / response.sensitivity;
        assert!(
            (ratio_05hz - 0.5).abs() < 0.05,
            "Response at 0.5 Hz should be ~50% of sensitivity, got {}",
            ratio_05hz
        );

        // At 10 Hz, response should be close to sensitivity
        let omega_10hz = 2.0 * PI * 10.0;
        let h_10hz = evaluate_response_at(&response, omega_10hz);
        let ratio_10hz = h_10hz.norm() / response.sensitivity;
        assert!(
            ratio_10hz > 0.99,
            "Response at 10 Hz should be ~100% of sensitivity, got {}",
            ratio_10hz
        );
    }

    #[test]
    fn test_deconvolve_response_5hz_sine() {
        // Test that deconvolution of a 5 Hz sine wave (at reference frequency)
        // gives values ≈ count / sensitivity
        let response = ChannelResponse {
            zeros: vec![(0.0, 0.0), (0.0, 0.0), (0.0, 0.0)],
            poles: vec![(-1.0, 0.0), (-3.03, 0.0), (-3.03, 0.0), (-666.67, 0.0)],
            normalization_factor: 673.744,
            stage_gain: 399_650_000.0,
            sensitivity: 399_650_000.0,
        };

        let sample_rate = 100.0;
        let n = 1000; // 10 seconds
        let amp = 10000.0; // 10000 counts amplitude

        // Generate 5 Hz sine (at reference frequency, where response is flat)
        let signal: Vec<f64> = (0..n)
            .map(|i| amp * (2.0 * PI * 5.0 * i as f64 / sample_rate).sin())
            .collect();

        let pre_filt = [0.1, 0.6, 0.95 * sample_rate, sample_rate];
        let deconv = deconvolve_response(&signal, &response, sample_rate, pre_filt, 4.5);

        // Expected amplitude: 10000 / 399650000 ≈ 2.502e-5
        let expected_amp = amp / response.sensitivity;

        // Measure amplitude in the middle (avoid edge effects)
        let mid = &deconv[n / 4..3 * n / 4];
        let max_val = mid.iter().map(|&x| x.abs()).fold(0.0_f64, f64::max);

        let ratio = max_val / expected_amp;
        assert!(
            (ratio - 1.0).abs() < 0.15,
            "At 5 Hz, deconvolved amplitude should match count/sensitivity. Got ratio={}, max={}, expected={}",
            ratio, max_val, expected_amp
        );
    }

    #[test]
    fn test_deconvolve_response_1hz_boost() {
        // Test that at 1 Hz, deconvolution gives LARGER values than simple division
        // because the instrument response is only ~81% of peak at 1 Hz
        let response = ChannelResponse {
            zeros: vec![(0.0, 0.0), (0.0, 0.0), (0.0, 0.0)],
            poles: vec![(-1.0, 0.0), (-3.03, 0.0), (-3.03, 0.0), (-666.67, 0.0)],
            normalization_factor: 673.744,
            stage_gain: 399_650_000.0,
            sensitivity: 399_650_000.0,
        };

        let sample_rate = 100.0;
        let n = 2000; // 20 seconds for good resolution at 1 Hz
        let amp = 10000.0;

        // Generate 1 Hz sine
        let signal: Vec<f64> = (0..n)
            .map(|i| amp * (2.0 * PI * 1.0 * i as f64 / sample_rate).sin())
            .collect();

        let pre_filt = [0.1, 0.6, 0.95 * sample_rate, sample_rate];
        let deconv = deconvolve_response(&signal, &response, sample_rate, pre_filt, 4.5);

        let simple_amp = amp / response.sensitivity;

        // Measure amplitude in the middle
        let mid = &deconv[n / 4..3 * n / 4];
        let max_val = mid.iter().map(|&x| x.abs()).fold(0.0_f64, f64::max);

        // At 1 Hz, the response is ~81% of peak, so deconvolution should give ~1.23x more
        let boost = max_val / simple_amp;
        assert!(
            boost > 1.1 && boost < 1.5,
            "At 1 Hz, deconvolved amplitude should be ~1.23x simple division. Got boost={}",
            boost
        );
    }

    #[test]
    fn test_pre_filter_weight() {
        let pf = [0.1, 0.6, 47.5, 50.0];

        assert_eq!(pre_filter_weight(0.0, &pf), 0.0);
        assert_eq!(pre_filter_weight(0.05, &pf), 0.0);
        assert!(pre_filter_weight(0.35, &pf) > 0.0 && pre_filter_weight(0.35, &pf) < 1.0);
        assert!((pre_filter_weight(1.0, &pf) - 1.0).abs() < 1e-10);
        assert!((pre_filter_weight(25.0, &pf) - 1.0).abs() < 1e-10);
        assert!(pre_filter_weight(48.0, &pf) > 0.0 && pre_filter_weight(48.0, &pf) < 1.0);
        assert_eq!(pre_filter_weight(50.0, &pf), 0.0);

        // With rsudp-style pre_filt [0.1, 0.6, 95, 100], all freqs up to Nyquist pass
        let pf_rsudp = [0.1, 0.6, 95.0, 100.0];
        assert!((pre_filter_weight(49.0, &pf_rsudp) - 1.0).abs() < 1e-10);
        assert!((pre_filter_weight(50.0, &pf_rsudp) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_deconvolve_spike_peak_preservation() {
        // Test that a broadband spike's peak is preserved by deconvolution.
        // Simulate a P-wave-like transient (3-sample Gaussian spike) in a background of noise.
        let response = ChannelResponse {
            zeros: vec![(0.0, 0.0), (0.0, 0.0), (0.0, 0.0)],
            poles: vec![(-1.0, 0.0), (-3.03, 0.0), (-3.03, 0.0), (-666.67, 0.0)],
            normalization_factor: 673.744,
            stage_gain: 399_650_000.0,
            sensitivity: 399_650_000.0,
        };

        let sample_rate = 100.0;
        let n = 2000; // 20 seconds
        let spike_amp = 100000.0; // Large spike in ADC counts
        let spike_center = n / 2; // Spike at center of window

        // Background: low-amplitude 3 Hz sine + 7 Hz sine (typical microseismic noise)
        let mut signal: Vec<f64> = (0..n)
            .map(|i| {
                let t = i as f64 / sample_rate;
                500.0 * (2.0 * PI * 3.0 * t).sin() + 300.0 * (2.0 * PI * 7.0 * t).sin()
            })
            .collect();

        // Add a broadband spike (Gaussian envelope * high-freq oscillation)
        for i in 0..20 {
            let offset = (i as f64 - 10.0) / 3.0;
            let gaussian = (-0.5 * offset * offset).exp();
            let osc = (2.0 * PI * 5.0 * i as f64 / sample_rate).sin();
            signal[spike_center - 10 + i] += spike_amp * gaussian * osc;
        }

        // Deconvolve with full window (simulating rsudp's approach)
        let pre_filt = [0.1, 0.6, 0.95 * sample_rate, sample_rate];
        let deconv_full = deconvolve_response(&signal, &response, sample_rate, pre_filt, 4.5);

        // Deconvolve with 512-sample context (simulating our live approach)
        let context_start = spike_center.saturating_sub(256);
        let context_end = (spike_center + 256).min(n);
        let context = &signal[context_start..context_end];
        let deconv_context = deconvolve_response(context, &response, sample_rate, pre_filt, 4.5);

        // Find peak in full deconvolution (near spike center)
        let search_start = spike_center - 50;
        let search_end = spike_center + 50;
        let peak_full = deconv_full[search_start..search_end]
            .iter().map(|&x| x.abs()).fold(0.0_f64, f64::max);

        // Find peak in context deconvolution (adjusted index)
        let ctx_spike_center = spike_center - context_start;
        let ctx_search_start = ctx_spike_center.saturating_sub(50);
        let ctx_search_end = (ctx_spike_center + 50).min(deconv_context.len());
        let peak_context = deconv_context[ctx_search_start..ctx_search_end]
            .iter().map(|&x| x.abs()).fold(0.0_f64, f64::max);

        // The context deconvolution should give a peak within 20% of the full deconvolution
        let ratio = peak_context / peak_full;
        eprintln!(
            "Spike peak test: full={:.6e}, context={:.6e}, ratio={:.3}",
            peak_full, peak_context, ratio
        );
        assert!(
            ratio > 0.8 && ratio < 1.2,
            "Context-window deconvolution peak should be within 20% of full-window. Got ratio={}",
            ratio
        );
    }
}
