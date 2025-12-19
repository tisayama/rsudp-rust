/// Recursive STA/LTA Trigger Algorithm.
///
/// Implements the recursive STA/LTA algorithm compatible with `obspy.signal.trigger.recursive_sta_lta`.
///
/// Formula (from obspy/signal/src/recstalta.c):
/// ```c
/// sta = csta * pow(a[i],2) + (1-csta)*sta;
/// lta = clta * pow(a[i],2) + (1-clta)*lta;
/// ```
#[derive(Debug, Clone)]
pub struct RecursiveStaLta {
    csta: f64,
    clta: f64,
    sta: f64,
    lta: f64,
    count: usize,
    nlta_len: usize,
}

impl RecursiveStaLta {
    /// Create a new RecursiveStaLta filter.
    pub fn new(nsta: usize, nlta: usize) -> Self {
        Self {
            csta: 1.0 / nsta as f64,
            clta: 1.0 / nlta as f64,
            sta: 0.0,
            lta: 0.0, 
            count: 0,
            nlta_len: nlta,
        }
    }

    /// Process a single sample and return the current STA/LTA ratio.
    /// Matches Obspy's C implementation exactly.
    pub fn process(&mut self, sample: f64) -> f64 {
        self.count += 1;

        // Obspy's C implementation starts the loop from i=1, skipping i=0 completely.
        if self.count == 1 {
            return 0.0;
        }

        // Handle NaN/Inf (Safety addition)
        if !sample.is_finite() {
            return 0.0;
        }

        let sq = sample * sample;
        
        // Exact formula and order from obspy/signal/src/recstalta.c
        self.sta = self.csta * sq + (1.0 - self.csta) * self.sta;
        self.lta = self.clta * sq + (1.0 - self.clta) * self.lta;
        
        // Obspy's Python wrapper or the C post-processing zeros out the first nlta samples.
        if self.count <= self.nlta_len {
            return 0.0;
        }

        self.sta / self.lta
    }

    /// Process a chunk of samples.
    pub fn process_chunk(&mut self, data: &[f64]) -> Vec<f64> {
        data.iter().map(|&x| self.process(x)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::path::Path;

    #[test]
    fn test_nan_handling() {
        let mut stalta = RecursiveStaLta::new(10, 100);
        let ratio = stalta.process(f64::NAN);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_compare_with_obspy_exact() {
        let script_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/scripts/generate_stalta_reference.py");
        let target_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
        if !target_dir.exists() {
            std::fs::create_dir(&target_dir).unwrap();
        }
        let output_csv = target_dir.join("reference.csv");
            
        let status = Command::new("python3")
            .arg(script_path)
            .arg(&output_csv)
            .status()
            .expect("Failed to execute python script");
            
        assert!(status.success());
        
        let mut rdr = csv::Reader::from_path(&output_csv).unwrap();
        let mut stalta = RecursiveStaLta::new(50, 200); 
        
        for (i, result) in rdr.records().enumerate() {
            let record = result.unwrap();
            let input: f64 = record[0].parse().unwrap();
            let expected_ratio: f64 = record[1].parse().unwrap();
            
            let ratio = stalta.process(input);
            
            let diff = (ratio - expected_ratio).abs();
            // Target high precision match (1e-12 or better)
            assert!(diff < 1e-12, 
                "Mismatch at index {}: got {:.20}, expected {:.20}, diff {:.20}", i, ratio, expected_ratio, diff);
        }
    }
}
