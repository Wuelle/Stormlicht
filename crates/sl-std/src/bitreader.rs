use error_derive::Error;

/// Wraps a byte buffer to allow reading individual bits
#[derive(Debug)]
pub struct BitReader<'a> {
    bytes: &'a [u8],
    pub byte_ptr: usize,
    pub bit_ptr: u8,
}

// this enum might grow once we add streaming (ie the reader wraps a Read instance)
#[derive(Clone, Copy, Debug, PartialEq, Error)]
pub enum Error {
    #[msg = "unexpected end of file"]
    UnexpectedEOF,

    #[msg = "attempting to read too many bits at once"]
    TooLargeRead,

    #[msg = "attempting to read bytes from an unaligned position"]
    UnalignedRead,
}

/// Create a bitmask for masking a range of bits in a byte
pub(crate) fn mask(from: u8, to: u8) -> u8 {
    assert!(from <= to);
    if to == 8 {
        if from == 8 {
            0
        } else {
            // shifting "to" out is valid but would cause an overflow
            !((1 << from) - 1)
        }
    } else {
        ((1 << to) - 1) & !((1 << from) - 1)
    }
}

impl<'a> BitReader<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            bytes: source,
            byte_ptr: 0,
            bit_ptr: 0,
        }
    }

    pub fn align_to_byte_boundary(&mut self) {
        if self.bit_ptr != 0 {
            self.bit_ptr = 0;
            self.byte_ptr += 1;
        }
    }

    pub fn read_bytes(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        if !self.bit_ptr == 0 {
            return Err(Error::UnalignedRead);
        } else if self.byte_ptr + buffer.len() > self.bytes.len() {
            return Err(Error::UnexpectedEOF);
        }

        buffer.copy_from_slice(&self.bytes[self.byte_ptr..self.byte_ptr + buffer.len()]);
        self.byte_ptr += buffer.len();

        Ok(())
    }

    pub fn num_consumed_bytes(&self) -> usize {
        if self.bit_ptr == 0 {
            self.byte_ptr
        } else {
            self.byte_ptr + 1
        }
    }

    pub fn read_single_bit(&mut self) -> Result<bool, Error> {
        self.read_bits::<u8>(1).map(|val| val == 1)
    }

    pub fn read_bits<T: From<u8> + std::ops::BitOrAssign<T> + std::ops::Shl<u8, Output = T>>(
        &mut self,
        mut bits_to_read: u8,
    ) -> Result<T, Error>
    where
        u8: Into<T>,
    {
        if std::mem::size_of::<T>() * 8 < bits_to_read as usize {
            return Err(Error::TooLargeRead);
        }

        let mut bits_available_from_current_byte = 8 - self.bit_ptr;

        let mut result = T::from(0);
        let mut bits_already_read = 0;

        while bits_to_read > bits_available_from_current_byte {
            let mask = mask(self.bit_ptr, 8);
            result |=
                ((self.bytes[self.byte_ptr] & mask) >> self.bit_ptr).into() << bits_already_read;

            let newly_read_bits = 8 - self.bit_ptr;

            bits_to_read -= newly_read_bits;
            bits_already_read += newly_read_bits;
            self.byte_ptr += 1;
            self.bit_ptr = 0;
            bits_available_from_current_byte = 8;
        }

        // read the remaining bits (guaranteed to be less than one byte)
        let mask = mask(self.bit_ptr, self.bit_ptr + bits_to_read);
        result |= ((self.bytes[self.byte_ptr] & mask) >> self.bit_ptr).into() << bits_already_read;
        self.bit_ptr += bits_to_read;

        if self.bit_ptr == 8 {
            self.bit_ptr = 0;
            self.byte_ptr += 1;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::BitReader;

    #[test]
    fn test_bitreader() {
        let bytes = [0b10010101, 0b00110011];
        let mut reader = BitReader::new(&bytes);

        assert_eq!(reader.read_bits::<u8>(4), Ok(0b0101));
        assert_eq!(reader.read_bits::<u8>(8), Ok(0b00111001));
        assert_eq!(reader.read_bits::<u8>(4), Ok(0b0011));
    }
}
