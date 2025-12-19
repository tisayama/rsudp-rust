use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use thiserror::Error;
use tracing::warn;

#[derive(Error, Debug)]
pub enum SteimError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid Steim code: {0}")]
    InvalidSteimCode(u8),
}

pub struct SteimDecoder;

impl SteimDecoder {
    pub fn decode_steim2(data: &[u8], num_samples: usize) -> Result<Vec<i32>, SteimError> {
        if num_samples == 0 { return Ok(Vec::new()); }

        let mut diffs = Vec::with_capacity(num_samples + 4);
        let mut rdr = Cursor::new(data);
        let mut x0 = 0;
        let mut xn = 0;
        let mut first_frame = true;

        // Total differences needed: num_samples (including the dummy d1 for offset)
        // Actually, many implementations extract all available and then truncate.
        'outer: while (rdr.position() as usize) < data.len() {
            let ctrl = match rdr.read_u32::<BigEndian>() {
                Ok(c) => c,
                Err(_) => break,
            };

            for i in 0..15 {
                let nibble = (ctrl >> (28 - i * 2)) & 0x03;
                let word = match rdr.read_u32::<BigEndian>() {
                    Ok(w) => w,
                    Err(_) => break 'outer,
                };

                if first_frame {
                    if i == 0 { x0 = word as i32; continue; }
                    if i == 1 { xn = word as i32; continue; }
                }

                match nibble {
                    0 => {} // Padding
                    1 => { // 4 x 8-bit
                        diffs.push(extract_bits(word, 24, 8));
                        diffs.push(extract_bits(word, 16, 8));
                        diffs.push(extract_bits(word, 8, 8));
                        diffs.push(extract_bits(word, 0, 8));
                    }
                    2 => { // Steim2 Nibble 2
                        let dn = word >> 30;
                        match dn {
                            1 => diffs.push(extract_bits(word, 0, 30)),
                            2 => {
                                diffs.push(extract_bits(word, 15, 15));
                                diffs.push(extract_bits(word, 0, 15));
                            }
                            3 => {
                                diffs.push(extract_bits(word, 20, 10));
                                diffs.push(extract_bits(word, 10, 10));
                                diffs.push(extract_bits(word, 0, 10));
                            }
                            _ => {}
                        }
                    }
                    3 => { // Steim2 Nibble 3
                        let dn = word >> 30;
                        match dn {
                            1 => { // 5 x 6-bit
                                diffs.push(extract_bits(word, 24, 6));
                                diffs.push(extract_bits(word, 18, 6));
                                diffs.push(extract_bits(word, 12, 6));
                                diffs.push(extract_bits(word, 6, 6));
                                diffs.push(extract_bits(word, 0, 6));
                            }
                            2 => { // 6 x 5-bit
                                diffs.push(extract_bits(word, 25, 5));
                                diffs.push(extract_bits(word, 20, 5));
                                diffs.push(extract_bits(word, 15, 5));
                                diffs.push(extract_bits(word, 10, 5));
                                diffs.push(extract_bits(word, 5, 5));
                                diffs.push(extract_bits(word, 0, 5));
                            }
                            3 => { // 7 x 4-bit
                                diffs.push(extract_bits(word, 24, 4));
                                diffs.push(extract_bits(word, 20, 4));
                                diffs.push(extract_bits(word, 16, 4));
                                diffs.push(extract_bits(word, 12, 4));
                                diffs.push(extract_bits(word, 8, 4));
                                diffs.push(extract_bits(word, 4, 4));
                                diffs.push(extract_bits(word, 0, 4));
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                // If we have enough diffs to reconstruct all samples (including skipped d1),
                // we should stop to avoid consuming padding as data.
                if diffs.len() > num_samples { break 'outer; }
            }
            first_frame = false;
        }

        let mut samples = Vec::with_capacity(num_samples);
        if num_samples > 0 {
            samples.push(x0);
            let mut cur = x0;
            
            // Reconstruct up to num_samples total
            for &d in diffs.iter().skip(1).take(num_samples - 1) {
                cur = cur.wrapping_add(d);
                samples.push(cur);
            }
            
            if let Some(&last) = samples.last() {
                if last != xn && samples.len() == num_samples {
                    warn!("Xn validation failed: expected {}, got {}. (len={}/{})", xn, last, samples.len(), num_samples);
                }
            }
        }
        Ok(samples)
    }

    pub fn decode_steim1(_data: &[u8], _num_samples: usize) -> Result<Vec<i32>, SteimError> {
        Ok(Vec::new())
    }
}

fn extract_bits(word: u32, shift: u32, bits: u32) -> i32 {
    let mask = (1u32 << bits) - 1;
    let val = (word >> shift) & mask;
    let sign_bit = 1u32 << (bits - 1);
    if val & sign_bit != 0 {
        (val | !(mask)) as i32
    } else {
        val as i32
    }
}
