use byteorder::{BigEndian, ReadBytesExt, ByteOrder};
use std::io::Cursor;
use thiserror::Error;
use tracing::warn;

#[derive(Error, Debug)]
pub enum SteimError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid control word")]
    InvalidControlWord,
    #[error("Invalid Steim sub-code: {0}")]
    InvalidSteimCode(u8),
    #[error("Xn validation failed: expected {expected}, got {actual}")]
    XnValidationFailed { expected: i32, actual: i32 },
}

pub struct SteimDecoder;

impl SteimDecoder {
    pub fn decode_steim1(data: &[u8], num_samples: usize) -> Result<Vec<i32>, SteimError> {
        Self::decode(data, num_samples, 1)
    }

    pub fn decode_steim2(data: &[u8], num_samples: usize) -> Result<Vec<i32>, SteimError> {
        Self::decode(data, num_samples, 2)
    }

    fn decode(data: &[u8], num_samples: usize, version: u8) -> Result<Vec<i32>, SteimError> {
        if num_samples == 0 {
            return Ok(Vec::new());
        }

        let mut diffs = Vec::with_capacity(num_samples + 10);
        let mut rdr = Cursor::new(data);
        
        let mut x0 = 0;
        let mut xn = 0;
        let mut first_frame = true;

        while (rdr.position() as usize) < data.len() {
            let control_word = match rdr.read_u32::<BigEndian>() {
                Ok(w) => w,
                Err(_) => break, // End of data
            };
            
            let mut keys = [0u8; 16];
            for i in 0..16 {
                keys[15 - i] = ((control_word >> (i * 2)) & 0x03) as u8;
            }

            for (i, &key) in keys.iter().enumerate().skip(1) {
                let word = match rdr.read_u32::<BigEndian>() {
                    Ok(w) => w,
                    Err(_) => break, // Partial frame?
                };

                if first_frame && i == 1 {
                    x0 = word as i32;
                    continue;
                }
                if first_frame && i == 2 {
                    xn = word as i32;
                    continue;
                }

                match key {
                    0 => { /* skip */ }
                    1 => {
                        let mut bytes = [0u8; 4];
                        BigEndian::write_u32(&mut bytes, word);
                        for b in bytes {
                            diffs.push(b as i8 as i32);
                        }
                    }
                    2 => {
                        if version == 1 {
                            // Steim1 Key 2: Two 2-byte differences
                            diffs.push((word >> 16) as i16 as i32);
                            diffs.push((word & 0xFFFF) as i16 as i32);
                        } else {
                            // Steim2 Key 2
                            let dn = (word >> 30) & 0x03;
                            match dn {
                                1 => diffs.push(extract_bits_signed(word, 30, 1)[0]),
                                2 => diffs.extend(extract_bits_signed(word, 15, 2)),
                                3 => diffs.extend(extract_bits_signed(word, 10, 3)),
                                _ => warn!("Invalid dn=0 for key=2 in Steim2"),
                            }
                        }
                    }
                    3 => {
                        if version == 1 {
                            // Steim1 Key 3: One 4-byte difference
                            diffs.push(word as i32);
                        } else {
                            // Steim2 Key 3
                            let dn = (word >> 30) & 0x03;
                            match dn {
                                0 => diffs.extend(extract_bits_signed(word, 6, 5)),
                                1 => diffs.extend(extract_bits_signed(word, 5, 6)),
                                2 => diffs.extend(extract_bits_signed(word, 4, 7)),
                                3 => diffs.extend(extract_bits_signed(word, 2, 15)), // MISSING EARLIER: 15x2-bit
                                _ => warn!("Invalid dn mapping for key=3 in Steim2"),
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
            if first_frame { first_frame = false; }
            
            let pos = rdr.position() as usize;
            if pos % 64 != 0 {
                let skip = 64 - (pos % 64);
                rdr.set_position((pos + skip) as u64);
            }
        }

        // Steim integration
        let mut samples = Vec::with_capacity(num_samples);
        samples.push(x0);
        let mut current = x0;
        
        for &diff in diffs.iter().skip(1) {
            if samples.len() >= num_samples {
                break;
            }
            current += diff;
            samples.push(current);
        }

        if samples.len() > 0 {
            let actual_xn = samples[samples.len() - 1];
            if samples.len() == num_samples && actual_xn != xn {
                return Err(SteimError::XnValidationFailed { expected: xn, actual: actual_xn });
            }
        }

        Ok(samples)
    }
}

fn extract_bits_signed(word: u32, bits: u8, count: usize) -> Vec<i32> {
    let mut out = Vec::with_capacity(count);
    let mask = if bits == 32 { 0xFFFFFFFF } else { (1u32 << bits) - 1 };
    for i in 0..count {
        let shift = (count - 1 - i) * (bits as usize);
        let val = (word >> shift) & mask;
        let sign_bit = 1u32 << (bits - 1);
        let signed_val = if (val & sign_bit) != 0 {
            if bits < 32 {
                (val as i32) - (1i32 << bits)
            } else {
                val as i32
            }
        } else {
            val as i32
        };
        out.push(signed_val);
    }
    out
}
