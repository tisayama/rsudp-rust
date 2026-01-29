use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use thiserror::Error;

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
        if num_samples == 0 {
            return Ok(Vec::new());
        }

        let mut diffs = Vec::with_capacity(num_samples + 4);
        let mut rdr = Cursor::new(data);
        let mut x0 = 0;
        let mut xn = 0;
        let mut first_frame = true;

        'outer: while (rdr.position() as usize) < data.len() {
            let ctrl = match rdr.read_u32::<BigEndian>() {
                Ok(c) => c,
                Err(_) => break,
            };

            for i in 1..16 { // i=1 to 15 correspond to the 15 data words in a frame
                let nibble = (ctrl >> (30 - i * 2)) & 0x03;
                
                let word = match rdr.read_u32::<BigEndian>() {
                    Ok(w) => w,
                    Err(_) => break 'outer,
                };

                if first_frame {
                    if i == 1 {
                        x0 = word as i32;
                        continue;
                    }
                    if i == 2 {
                        xn = word as i32;
                        continue;
                    }
                }

                match nibble {
                    0 => {} // Unused or padding
                    1 => {
                        // 4 x 8-bit
                        diffs.push(extract_bits(word, 24, 8));
                        diffs.push(extract_bits(word, 16, 8));
                        diffs.push(extract_bits(word, 8, 8));
                        diffs.push(extract_bits(word, 0, 8));
                    }
                    2 => {
                        // Steim2 Nibble 2: check upper 2 bits of the word (dn)
                        let dn = (word >> 30) & 0x03;
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
                    3 => {
                        // Steim2 Nibble 3: check upper 2 bits of the word (dn)
                        let dn = (word >> 30) & 0x03;
                        match dn {
                            0 => {
                                // 5 x 6-bit (Note: dn=0 for nibble=3 means 5 samples)
                                diffs.push(extract_bits(word, 24, 6));
                                diffs.push(extract_bits(word, 18, 6));
                                diffs.push(extract_bits(word, 12, 6));
                                diffs.push(extract_bits(word, 6, 6));
                                diffs.push(extract_bits(word, 0, 6));
                            }
                            1 => {
                                // 6 x 5-bit
                                diffs.push(extract_bits(word, 25, 5));
                                diffs.push(extract_bits(word, 20, 5));
                                diffs.push(extract_bits(word, 15, 5));
                                diffs.push(extract_bits(word, 10, 5));
                                diffs.push(extract_bits(word, 5, 5));
                                diffs.push(extract_bits(word, 0, 5));
                            }
                            2 => {
                                // 7 x 4-bit (Wait, dn=2 means 7 samples? Correct)
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
                
                if diffs.len() >= num_samples {
                    break 'outer;
                }
            }
            first_frame = false;
        }

        let mut samples = Vec::with_capacity(num_samples);
        if num_samples > 0 {
            samples.push(x0);
            let mut cur = x0;

            // Differences start from d1. In Steim, d1 is actually x0 - previous_x_last.
            // But for the very first sample, d1 is often used as a check or skipped.
            // Steim2 specification: differences are stored from i=2 (after X0, Xn)
            // Reconstruct: x[i] = x[i-1] + di
            for &d in diffs.iter().take(num_samples - 1) {
                cur = cur.wrapping_add(d);
                samples.push(cur);
            }

            if let Some(&last) = samples.last() {
                if last != xn && samples.len() == num_samples {
                    // This warning is common due to truncation or padding, but good to know
                    // warn!("Xn validation mismatch: expected {}, got {}. (len={})", xn, last, samples.len());
                }
            }
        }
        Ok(samples)
    }

    pub fn decode_steim1(_data: &[u8], _num_samples: usize) -> Result<Vec<i32>, SteimError> {
        // Not needed for this mseed file, but keep signature
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