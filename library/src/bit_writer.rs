use thiserror::Error;

pub(crate) struct BitWriter {
    byte_vector: Vec<u8>,
    bit_index: usize,
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            byte_vector: Vec::new(),
            bit_index: 0,
        }
    }

    pub fn write_bits8(&mut self, bits: usize, value: u8) -> Result<(), BitWriterError> {
        if bits > 8 {
            return Err(BitWriterError::TooManyBits(bits, 8));
        }
        if let Some(h) = value.highest_one() {
            let h = h as usize;
            if h >= bits {
                return Err(BitWriterError::TooManyBitsInValue(h, bits));
            }
        }
        if bits == 0 {
            return Ok(());
        }

        let remaining = 8 - bits;
        let index = self.byte_vector.len().max(1) - 1;

        if self.bit_index == 0 {
            self.byte_vector.push(value << remaining);
            self.bit_index = bits % 8;
            return Ok(());
        } else if self.bit_index < remaining {
            self.byte_vector[index] |= value << (remaining - self.bit_index);
        } else if self.bit_index == remaining {
            self.byte_vector[index] |= value;
        } else {
            let after = self.bit_index + bits - 8;
            self.byte_vector[index] |= value >> after;
            self.byte_vector.push(value << (8 - after));
        }

        self.bit_index = (self.bit_index + bits) % 8;

        Ok(())
    }

    pub fn get_buffer<'a>(&'a self) -> &'a [u8] {
        &self.byte_vector
    }

    pub fn get_vector(self) -> Vec<u8> {
        self.byte_vector
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Error)]
pub enum BitWriterError {
    #[error("Too many bits ({0}) for function with maximum {1} bits")]
    TooManyBits(usize, usize),

    #[error("More bits sent than specified ({0} > {1})")]
    TooManyBitsInValue(usize, usize),
}
