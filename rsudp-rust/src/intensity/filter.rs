use rustfft::{FftPlanner, num_complex::Complex};

pub struct JmaFilter {
    sample_rate: f64,
}

impl JmaFilter {
    pub fn new(sample_rate: f64) -> Self {
        Self { sample_rate }
    }

    fn calculate_gain(&self, f: f64) -> f64 {
        if f <= 0.0 {
            return 0.0;
        }

        // 1. Period effect filter (sqrt(1/f))
        let period_gain = (1.0 / f).sqrt();

        // 2. High-cut filter
        // H(f) = 1 / sqrt(1 + 0.694x^2 + 0.241x^4 + 0.0557x^6 + 0.009664x^8 + 0.00134x^10 + 0.000155x^12)
        let x = f / 10.0;
        let x2 = x * x;
        let x4 = x2 * x2;
        let x6 = x4 * x2;
        let x8 = x4 * x4;
        let x10 = x6 * x4;
        let x12 = x6 * x6;
        let high_cut = 1.0
            / (1.0
                + 0.694 * x2
                + 0.241 * x4
                + 0.0557 * x6
                + 0.009664 * x8
                + 0.00134 * x10
                + 0.000155 * x12)
                .sqrt();

        // 3. Low-cut filter
        let low_cut = (1.0 - (-(f / 0.5).powi(3)).exp()).sqrt();

        period_gain * high_cut * low_cut
    }

    pub fn calculate_intensity(&self, x: &[f64], y: &[f64], z: &[f64]) -> f64 {
        let n = x.len();
        if n == 0 {
            return 0.0;
        }

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(n);

        let mut process_component = |data: &[f64]| -> Vec<f64> {
            // Demean the window
            let mean = data.iter().sum::<f64>() / n as f64;
            let mut buffer: Vec<Complex<f64>> = data
                .iter()
                .map(|&v| Complex {
                    re: v - mean,
                    im: 0.0,
                })
                .collect();

            fft.process(&mut buffer);

            for i in 0..n {
                let f = if i <= n / 2 {
                    i as f64 * self.sample_rate / n as f64
                } else {
                    (n - i) as f64 * self.sample_rate / n as f64
                };
                let gain = self.calculate_gain(f);
                buffer[i] *= gain;
            }

            let ifft = planner.plan_fft_inverse(n);
            ifft.process(&mut buffer);

            // Correct normalization
            buffer.iter().map(|c| c.re / n as f64).collect()
        };

        let fx = process_component(x);
        let fy = process_component(y);
        let fz = process_component(z);

        // Vector composition
        let mut composite: Vec<f64> = fx
            .iter()
            .zip(fy.iter())
            .zip(fz.iter())
            .map(|((&ax, &ay), &az)| (ax * ax + ay * ay + az * az).sqrt())
            .collect();

        // JMA Intensity Algorithm:
        // Value exceeded for exactly 0.3 seconds.
        composite.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        // If sample_rate is 100Hz, we need the 30th sample (0-indexed: index 29 or 30).
        // Official JMA: "The value that is exceeded for a cumulative duration of 0.3s"
        // This means we take the (0.3 * sample_rate)-th largest value.
        let idx = (0.3 * self.sample_rate).round() as usize;
        let a_03 = if idx < composite.len() {
            composite[idx]
        } else {
            composite.last().cloned().unwrap_or(0.0)
        };

        if a_03 > 0.0 {
            // I = 2.0 * log10(a) + 0.94
            // Final truncation to 1 decimal place is common for display,
            // but for exact matching we keep precision.
            2.0 * a_03.log10() + 0.94
        } else {
            0.0
        }
    }
}
