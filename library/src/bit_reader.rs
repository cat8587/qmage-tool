use thiserror::Error;

pub(crate) struct BitReader<'a> {
    buffer: &'a [u8],
    bit_offset: usize,
    byte_offset: usize,
}

impl<'a> BitReader<'a> {
    pub fn new_safe(buffer: &'a [u8]) -> Result<Self, BitReaderError> {
        if buffer.len() >= (usize::MAX) / 8 - 10 {
            Err(BitReaderError::IntegerOverflow)
        } else {
            Ok(Self {
                buffer,
                bit_offset: 0,
                byte_offset: 0,
            })
        }
    }

    pub fn new(buffer: &'a [u8]) -> Self {
        Self::new_safe(buffer).unwrap()
    }

    pub fn check_remaining_bits(&self, bits: usize) -> Result<(), BitReaderError> {
        if (8 * self.byte_offset + self.bit_offset + bits) > 8 * self.buffer.len() {
            Err(BitReaderError::RanOut(
                bits,
                (8 * self.buffer.len()) - (8 * self.byte_offset + self.bit_offset),
            ))
        } else {
            Ok(())
        }
    }

    pub fn read_bits8_safe(&mut self, bits: usize) -> Result<u8, BitReaderError> {
        if bits == 0 {
            return Ok(0);
        } else if bits > 8 {
            return Err(BitReaderError::RequestTooLarge(bits, "u8"));
        }
        let _ = self.check_remaining_bits(bits)?;
        let mut byte_a = self.buffer[self.byte_offset];
        let bits_read = 8 - self.bit_offset;
        byte_a <<= self.bit_offset;
        byte_a >>= self.bit_offset;
        let difference = (bits as isize) - (bits_read as isize);
        if difference < 0 {
            byte_a >>= (-difference) as usize;
            self.bit_offset += bits;
        } else if difference > 0 {
            self.byte_offset += 1;
            let mut byte_b = self.buffer[self.byte_offset];
            byte_a <<= difference as usize;
            byte_b >>= (8 - difference) as usize;
            byte_a |= byte_b;
            self.bit_offset = difference as usize;
        } else {
            self.byte_offset += 1;
            self.bit_offset = 0;
        };

        Ok(byte_a)
    }

    pub fn read_bits8(&mut self, bits: usize) -> u8 {
        self.read_bits8_safe(bits).unwrap()
    }

    pub fn read_bits16_safe(&mut self, bits: usize) -> Result<u16, BitReaderError> {
        if bits == 0 {
            return Ok(0);
        } else if bits > 16 {
            return Err(BitReaderError::RequestTooLarge(bits, "u16"));
        } else if bits <= 8 {
            return self.read_bits8_safe(bits).map(|n| n as u16);
        }

        let _ = self.check_remaining_bits(bits)?;

        let a = self.read_bits8_safe(bits - 8)? as u16;
        let b = self.read_bits8_safe(8)? as u16;
        Ok((a << 8) | b)
    }

    pub fn read_bits16(&mut self, bits: usize) -> u16 {
        self.read_bits16_safe(bits).unwrap()
    }

    pub fn read_2_bytes_le(&mut self) -> u16 {
        let word = self.read_bits16(16);
        (word << 8) | (word >> 8)
    }
}

#[derive(Error, Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum BitReaderError {
    #[error("Requested {0} bits, too much for a {1} type")]
    RequestTooLarge(usize, &'static str),

    #[error("Ran out of bits, requested {0} while only {1} were left")]
    RanOut(usize, usize),

    #[error("Integer overflow while calculating remaining bits")]
    IntegerOverflow,
}
