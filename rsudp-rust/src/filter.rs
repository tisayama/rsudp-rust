use crate::trigger::Biquad;
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
    let mut sections = Vec::new();

    for k in 0..order {
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
        // For pole s1: z-domain pole at z = (1 + s/(2*fs)) / (1 - s/(2*fs))
        // We form two 2nd-order sections, one for each conjugate pair

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bandpass_basic_properties() {
        // Order 4, 0.7-2.0 Hz, fs=100
        let sections = butter_bandpass_sos(4, 0.7, 2.0, 100.0);
        // Bandpass order = 2 * lowpass_order = 8, so 4 biquad sections
        assert_eq!(sections.len(), 8);

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
}
