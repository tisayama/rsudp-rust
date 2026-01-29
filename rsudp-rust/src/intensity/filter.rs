use rustfft::{FftPlanner, num_complex::Complex};

pub struct JmaFilter {
    sample_rate: f64,
}

impl JmaFilter {
    pub fn new(sample_rate: f64) -> Self {
        Self { sample_rate }
    }

    /// Official JMA Frequency Weighting Characteristics (1996)
    fn calculate_gain(&self, f: f64) -> f64 {
        if f <= 0.0 {
            return 0.0;
        }

        // 1. Period effect filter (A_1(f))
        let x = 1.0 / f; // period T
        let period_gain = (x / (1.0 + 0.694 * x.powi(2) + 0.241 * x.powi(4) + 0.0557 * x.powi(6) 
                          + 0.009664 * x.powi(8) + 0.00134 * x.powi(10) + 0.000155 * x.powi(12))).sqrt();

        // 2. High-cut filter (A_2(f))
        let f_h = f / 10.0;
        let high_cut = 1.0 / (1.0 + f_h.powi(6)).sqrt();

        // 3. Low-cut filter (A_3(f))
        // This is the official low-cut specified by JMA. 
        // No additional hard cutoffs should be applied here to maintain theoretical correctness.
        let f_l = f / 0.5;
        let low_cut = (1.0 - (-f_l.powi(3)).exp()).sqrt();

        let gain = period_gain * high_cut * low_cut;
        if gain.is_nan() || gain.is_infinite() { 0.0 } else { gain }
    }

    /// Standard Cosine Taper to prevent spectral leakage in finite-window FFT.
    /// This is a common signal processing technique and does not violate JMA theory.
    fn apply_taper(data: &mut [f64], percentage: f64) {
        let n = data.len();
        let taper_len = (n as f64 * percentage).round() as usize;
        if taper_len == 0 { return; }
        
        for i in 0..taper_len {
            let f = 0.5 * (1.0 - (std::f64::consts::PI * i as f64 / taper_len as f64).cos());
            data[i] *= f;
            data[n - 1 - i] *= f;
        }
    }

    pub fn calculate_intensity(&self, x: &[f64], y: &[f64], z: &[f64]) -> f64 {
        let n = x.len();
        if n == 0 { return -2.0; }

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(n);

        let mut process_component = |data: &[f64]| -> Vec<f64> {
            let mut d = data.to_vec();
            let dn = d.len() as f64;
            
            // 1. Robust Linear Detrend
            // Removing DC offset and linear drift is required for FFT stability 
            // especially when huge gravity offset (1g) exists on ENZ.
            let sum_x: f64 = (0..d.len()).map(|i| i as f64).sum();
            let sum_x2: f64 = (0..d.len()).map(|i| (i as f64).powi(2)).sum();
            let sum_y: f64 = d.iter().sum();
            let sum_xy: f64 = d.iter().enumerate().map(|(i, &v)| i as f64 * v).sum();
            
            let denom = dn * sum_x2 - sum_x.powi(2);
            if denom.abs() > 1e-9 {
                let slope = (dn * sum_xy - sum_x * sum_y) / denom;
                let intercept = (sum_y - slope * sum_x) / dn;
                for i in 0..d.len() {
                    d[i] -= slope * i as f64 + intercept;
                }
            }
            
            // 2. Apply Taper (5% on each side)
            // This suppresses the "edge effects" that cause spikes in noise data.
            Self::apply_taper(&mut d, 0.05);

            let mut buffer: Vec<Complex<f64>> = d
                .iter()
                .map(|&v| Complex { re: v, im: 0.0 })
                .collect();

            fft.process(&mut buffer);

            for i in 0..n {
                let f = if i <= n / 2 {
                    i as f64 * self.sample_rate / dn
                } else {
                    (n - i) as f64 * self.sample_rate / dn
                };
                let gain = self.calculate_gain(f);
                buffer[i] *= gain;
            }

            let ifft = planner.plan_fft_inverse(n);
            ifft.process(&mut buffer);

            buffer.iter().map(|c| c.re / dn).collect()
        };

        let fx = process_component(x);
        let fy = process_component(y);
        let fz = process_component(z);

        let mut composite: Vec<f64> = fx
            .iter()
            .zip(fy.iter())
            .zip(fz.iter())
            .map(|((&ax, &ay), &az)| (ax * ax + ay * ay + az * az).sqrt())
            .collect();

        // 3. Sort to find the acceleration 'a' where duration is >= 0.3s
        composite.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        let idx = (0.3 * self.sample_rate).round() as usize;
        let a_03 = if idx < composite.len() {
            composite[idx]
        } else {
            composite.last().cloned().unwrap_or(0.0)
        };

        // 4. Calculate intensity without arbitrary thresholds
        if a_03 > 1e-15 { // Minimal guard for log10
            let intensity = 2.0 * a_03.log10() + 0.94;
            // Floor to 1 decimal place as per JMA standard for display
            let result = (intensity * 10.0).floor() / 10.0;
            if result < -2.0 { -2.0 } else { result }
        } else {
            -2.0
        }
    }
}