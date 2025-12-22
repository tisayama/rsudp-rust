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

        // 1. Period effect filter (JMA standard)
        // Re-implementing the standard JMA filter response
        let x = 1.0 / f; // period T
        let period_gain = (x / (1.0 + 0.694 * x.powi(2) + 0.241 * x.powi(4) + 0.0557 * x.powi(6) 
                          + 0.009664 * x.powi(8) + 0.00134 * x.powi(10) + 0.000155 * x.powi(12))).sqrt();

        // 2. High-cut filter (Butterworth-like)
        let f_h = f / 10.0;
        let high_cut = 1.0 / (1.0 + f_h.powi(6)).sqrt();

        // 3. Low-cut filter
        let f_l = f / 0.5;
        let low_cut = (1.0 - (-f_l.powi(3)).exp()).sqrt();

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

            buffer.iter().map(|c| c.re / n as f64).collect()
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

        composite.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        let idx = (0.3 * self.sample_rate).round() as usize;
        let a_03 = if idx < composite.len() {
            composite[idx]
        } else {
            composite.last().cloned().unwrap_or(0.0)
        };

        if a_03 > 0.0 {
            // I = 2.0 * log10(a) + 0.94
            let intensity = 2.0 * a_03.log10() + 0.94;
            // Floor to 1 decimal place as per JMA standard for display (e.g. 3.07 -> 3.0)
            (intensity * 10.0).floor() / 10.0
        } else {
            0.0
        }
    }
}