use std::f64::consts::PI;
use num_complex::Complex;

// Helper to create 4th order (2 cascaded 2nd order) Butterworth bandpass SOS
// Equivalent to scipy.signal.butter(4, [low, high], btype='band', output='sos', fs=fs)
// Note: "order 4" in scipy for bandpass means 4th order -> 8 poles total -> 4 biquad sections
pub fn butter_bandpass_sos(order: usize, low_freq: f64, high_freq: f64, fs: f64) -> Vec<Biquad> {
    if order != 4 {
        panic!("Only order 4 implementation is supported for now (matches rsudp default)");
    }

    let nyquist = fs / 2.0;
    let low = low_freq / nyquist;
    let high = high_freq / nyquist;

    // Pre-warp frequencies
    let u_low = (PI * low / 2.0).tan();
    let u_high = (PI * high / 2.0).tan();
    
    let bw = u_high - u_low;
    let center_sq = u_high * u_low;

    // Analog prototype (Butterworth, order 4)
    // Poles on unit circle in s-plane: exp(j * (2k + n + 1) * pi / (2n))
    // For n=4: k=0..3 -> angles: 5pi/8, 7pi/8, 9pi/8, 11pi/8
    // We only need poles with real part < 0 (LHP)
    let angles = vec![
        5.0 * PI / 8.0,
        7.0 * PI / 8.0,
        9.0 * PI / 8.0,
        11.0 * PI / 8.0,
    ];

    let mut sections = Vec::new();

    // Group poles into conjugate pairs to form 2nd order sections
    // Angles 5pi/8 and 11pi/8 are conjugates (11pi/8 = -5pi/8)
    // Angles 7pi/8 and 9pi/8 are conjugates (9pi/8 = -7pi/8)
    
    // We process pairs of poles from the prototype
    // For bandpass transformation, each real pole becomes 1 biquad, each conjugate pair becomes 2 biquads (4th order total)
    // But here we start with 4th order prototype, so we have 4 poles in LHP.
    // Each pair (p, p*) transforms into a 4th order section (2 biquads) in bandpass?
    // Wait, scipy 'order 4' bandpass means the RESULT is 8th order (4 biquads).
    // Let's stick to the standard bilinear transform steps.

    let poles_proto: Vec<Complex<f64>> = angles.iter().map(|&a| Complex::from_polar(1.0, a)).collect();

    // Bandpass transformation: s -> (s^2 + center_sq) / (s * bw)
    // Solve for s: s^2 - p*bw*s + center_sq = 0
    let mut poles_analog = Vec::new();
    for p in poles_proto {
        // Roots of s^2 - (p*bw)*s + center_sq = 0
        let b_val = -p * bw;
        let c_val = Complex::new(center_sq, 0.0);
        let disc = (b_val * b_val - 4.0 * c_val).sqrt();
        let s1 = (-b_val + disc) / 2.0;
        let s2 = (-b_val - disc) / 2.0;
        poles_analog.push(s1);
        poles_analog.push(s2);
    }
    
    // We have 8 poles. Group into 4 conjugate pairs.
    // Sort by real part? Or just find conjugates.
    // Since we generated them from conjugates, s1 corresponding to p and s1 corresponding to p* should be conjugates?
    // Actually, simpler approach for Biquads:
    // Bilinear transform: z = (1+s)/(1-s) -> s = (z-1)/(z+1)
    
    // Let's map poles to z-plane first
    let mut poles_z = Vec::new();
    for s in poles_analog {
        let z = (1.0 + s) / (1.0 - s);
        poles_z.push(z);
    }

    // Zeros: For bandpass, we have N zeros at z=1 and N zeros at z=-1 (from s=0 and s=inf)
    // N = order (4)
    // So 4 zeros at +1, 4 zeros at -1.
    
    // Group into sections. Each section needs 2 poles and 2 zeros.
    // We need to pair complex conjugate poles.
    
    // Simple greedy pairing of conjugates
    let mut used = vec![false; poles_z.len()];
    for i in 0..poles_z.len() {
        if used[i] { continue; }
        
        // Find conjugate
        let mut best_j = i;
        let mut min_err = 1e9;
        
        for j in (i+1)..poles_z.len() {
            if used[j] { continue; }
            let err = (poles_z[i].re - poles_z[j].re).abs() + (poles_z[i].im + poles_z[j].im).abs();
            if err < min_err {
                min_err = err;
                best_j = j;
            }
        }
        
        used[i] = true;
        used[best_j] = true;
        
        let p1 = poles_z[i];
        let p2 = poles_z[best_j];
        
        // Form Biquad from poles p1, p2 and zeros +1, -1
        // (z - 1)(z + 1) = z^2 - 1
        // (z - p1)(z - p2) = z^2 - (p1+p2)z + p1p2
        
        let poly_p_re = - (p1 + p2).re;
        let poly_p_abs_sq = (p1 * p2).re; // Assuming conjugates, product is real
        
        // Denominator (a): 1, poly_p_re, poly_p_abs_sq
        // Numerator (b): 1, 0, -1 (from z^2 - 1)
        
        // Apply gain? Usually done globally or distributed. 
        // For Butterworth bandpass, max gain is at center.
        // We need to normalize so gain is 1.0 at band center.
        // Or simpler: normalize at DC or Nyquist? No, bandpass is 0 there.
        // Standard approach: normalize so sum(a) = sum(b)? No.
        // Let's use the standard biquad form directly.
        
        // Direct definition:
        // H(z) = (b0 + b1 z^-1 + b2 z^-2) / (1 + a1 z^-1 + a2 z^-2)
        // a0 is normalized to 1.
        
        let a0 = 1.0;
        let a1 = poly_p_re;
        let a2 = poly_p_abs_sq;
        
        let b0 = 1.0;
        let b1 = 0.0;
        let b2 = -1.0;
        
        // Gain adjustment to match scipy behavior (optimally distributed)
        // Scipy distributes gain. For a single section bandpass biquad (z^2-1)/(z^2+a1z+a2):
        // Gain at center freq?
        // Let's calculate gain at a reference frequency in the passband (e.g. geometric mean)
        // But doing this per section is complex.
        // Alternative: Use the s-plane to z-plane substitution formula directly for Biquad coefficients
        
        sections.push(Biquad { b0, b1, b2, a1, a2, x1:0.0, x2:0.0, y1:0.0, y2:0.0 });
    }
    
    // Calculate global gain to normalize peak to 1.0
    // Evaluate transfer function at center frequency
    let center_freq = (low_freq * high_freq).sqrt();
    let omega = 2.0 * PI * center_freq / fs;
    let z = Complex::from_polar(1.0, omega);
    
    let mut mag = 1.0;
    for s in &sections {
        let num = s.b0 * z * z + s.b1 * z + s.b2;
        let den = z * z + s.a1 * z + s.a2;
        mag *= (num / den).norm();
    }
    
    // Distribute 1/mag gain to all sections (nth root)
    let section_gain = (1.0 / mag).powf(1.0 / sections.len() as f64);
    
    for s in &mut sections {
        s.b0 *= section_gain;
        s.b1 *= section_gain;
        s.b2 *= section_gain;
    }

    sections
}
