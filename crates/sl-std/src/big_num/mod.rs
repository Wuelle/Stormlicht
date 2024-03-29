use std::{iter, mem, ops};

cfg_match! {
    cfg(target_pointer_width = "64") => {
        pub type Digit = u64;
        pub type BigDigit = u128;
    }
    cfg(target_pointer_width = "32") => {
        pub type Digit = u32;
        pub type BigDigit = u64;
    }
    _ => {
        compile_error!("Arbitrary sized integers are only available for 32/64 bit platforms");
    }
}

const BYTES_PER_DIGIT: usize = mem::size_of::<Digit>();

#[macro_export]
macro_rules! bignum {
    ($n: literal) => {
        $crate::big_num::BigNum::new(stringify!($n))
    };
}

const POWERS: [(Digit, usize); 256] = {
    let mut powers = [(0, 0); 256];

    let mut radix = 2;
    while radix < 256 {
        let mut power = 1;
        let mut base: Digit = radix;

        while let Some(new_base) = base.checked_mul(radix) {
            base = new_base;
            power += 1;
        }

        powers[radix as usize] = (base, power);
        radix += 1;
    }

    powers
};

/// A dynamically sized unsigned integer type
#[derive(Clone, Debug)]
pub struct BigNum {
    /// The least significant digit comes first
    digits: Vec<Digit>,
}

impl BigNum {
    #[must_use]
    pub fn new(number: &str) -> Self {
        if let Some(without_prefix) = number.strip_prefix("0b") {
            return Self::new_with_radix(&to_radix(without_prefix, 2), 2);
        }

        if let Some(without_prefix) = number.strip_prefix("0o") {
            return Self::new_with_radix(&to_radix(without_prefix, 8), 8);
        }

        if let Some(without_prefix) = number.strip_prefix("0x") {
            return Self::new_with_radix(&to_radix(without_prefix, 16), 16);
        }

        Self::new_with_radix(&to_radix(number, 10), 10)
    }

    /// Utility function for the [BigNum] value `0`
    #[inline]
    #[must_use]
    pub fn zero() -> Self {
        Self::from_digits(vec![0])
    }

    #[inline]
    #[must_use]
    pub fn from_digits(digits: Vec<Digit>) -> Self {
        Self { digits }
    }

    /// Parse from big-endian digits
    pub fn new_with_radix(digits: &[u32], radix: u32) -> Self {
        if digits.is_empty() {
            return Self::zero();
        }

        assert!(digits.iter().all(|&v| v < radix));

        // Split the digits into chunks
        let (base, power) = POWERS[radix as usize];

        let head_len = if digits.len() % power == 0 {
            power
        } else {
            digits.len() % power
        };

        let (head, tail) = digits.split_at(head_len);

        let mut result = Self::from_digits(vec![]);
        let first = head.iter().fold(0, |acc, &digit| {
            acc * Digit::from(radix) + Digit::from(digit)
        });
        result.digits.push(first);

        let exact_chunks = tail.chunks_exact(power);
        debug_assert!(exact_chunks.remainder().is_empty());

        for chunk in exact_chunks {
            result.digits.push(0);

            let mut carry: BigDigit = 0;
            for d in result.digits_mut() {
                carry += BigDigit::from(*d) * BigDigit::from(base);
                *d = carry as Digit;
                carry >>= Digit::BITS;
            }

            assert_eq!(carry, 0);

            result = result + Digit::from(chunk.iter().fold(0, |acc, &digit| acc * radix + digit));
        }

        result
    }

    /// Try to shrink the internal vector as much as possible by
    /// deallocating unused capacity and removing leading zeros.
    #[inline]
    pub fn compact(&mut self) {
        self.digits.truncate(self.last_nonzero_digit() + 1);
    }

    #[inline]
    #[must_use]
    fn digits(&self) -> &[Digit] {
        &self.digits
    }

    /// List of digits, with leading zeros removed
    #[inline]
    #[must_use]
    fn nonzero_digits(&self) -> &[Digit] {
        &self.digits()[..=self.last_nonzero_digit()]
    }

    #[inline]
    #[must_use]
    fn digits_mut(&mut self) -> &mut [Digit] {
        &mut self.digits
    }

    fn last_nonzero_digit(&self) -> usize {
        self.digits()
            .iter()
            .enumerate()
            .rev()
            .find_map(
                |(index, digit)| {
                    if *digit == 0 {
                        None
                    } else {
                        Some(index)
                    }
                },
            )
            .unwrap_or_default()
    }

    pub fn from_be_bytes(bytes: &[u8]) -> Self {
        let skip_from_start = bytes.len() % BYTES_PER_DIGIT;
        let n_digits = (bytes.len() + BYTES_PER_DIGIT - 1) / BYTES_PER_DIGIT;
        let mut digits = Vec::with_capacity(n_digits);

        // Skip the first n bytes such that the remainder is evenly divisible into digits
        let chunks = bytes[skip_from_start..]
            .array_chunks::<BYTES_PER_DIGIT>()
            .rev()
            .copied();

        for chunk in chunks {
            digits.push(Digit::from_be_bytes(chunk));
        }

        // Take care of the most significant bytes we skipped earlier
        if skip_from_start != 0 {
            let mut buffer = [0; BYTES_PER_DIGIT];
            buffer[BYTES_PER_DIGIT - skip_from_start..].copy_from_slice(&bytes[..skip_from_start]);
            digits.push(Digit::from_be_bytes(buffer));
        }

        Self::from_digits(digits)
    }
}

impl ops::Add for BigNum {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        // Attempt to reuse the storage from the larger argument
        let (mut destination, other) = if self.digits.capacity() < other.digits.capacity() {
            (other, self)
        } else {
            (self, other)
        };

        let max_digits = if destination.digits().len() < other.digits().len() {
            // Reserve the maximum space that the result can take up
            // This might not be a reallocation since we chose the
            // vector with a larger capacity earlier.
            destination.digits.resize(other.digits.len() + 1, 0);
            other.digits.len()
        } else {
            destination.digits.len()
        };

        let mut carry = false;
        for (d1, &d2) in destination
            .digits
            .iter_mut()
            .zip(other.digits().iter().chain(iter::repeat(&0)))
            .take(max_digits)
        {
            (*d1, carry) = d1.carrying_add(d2, carry);
        }

        // We resized with zero before so there can't be a carry afterwards
        debug_assert!(!carry);

        destination.compact(); // if we allocated too much, free the space
        destination
    }
}

impl ops::Add<Digit> for BigNum {
    type Output = Self;

    fn add(mut self, other: Digit) -> Self::Output {
        let mut carry = false;
        for digit in self.digits.iter_mut() {
            (*digit, carry) = digit.carrying_add(other, carry);

            if !carry {
                return self;
            }
        }

        if carry {
            self.digits.push(1);
        }

        self
    }
}

impl ops::Shl<usize> for &BigNum {
    type Output = BigNum;

    fn shl(self, rhs: usize) -> Self::Output {
        // First shift by digits
        let n_digits_to_insert = rhs / Digit::BITS as usize;
        let final_length = n_digits_to_insert + self.nonzero_digits().len();
        let mut digits = vec![0; final_length];
        digits[n_digits_to_insert..].copy_from_slice(self.nonzero_digits());

        // Then, shift by the remainder (which will always be less than the size of a digit)
        let remainder = rhs % Digit::BITS as usize;
        if remainder != 0 {
            let mut carry: Digit = 0;
            for digit in &mut digits {
                let new_carry = *digit >> (Digit::BITS as usize - remainder);
                *digit = (*digit << remainder) | carry;
                carry = new_carry;
            }
            if carry != 0 {
                digits.push(carry);
            }
        }

        BigNum::from_digits(digits)
    }
}

impl PartialEq for BigNum {
    fn eq(&self, other: &Self) -> bool {
        self.nonzero_digits() == other.nonzero_digits()
    }
}

// Takes an ascii string and converts it to a sequence of digits in the given
// radix and removes leading zeros So `"01_23F"` in base 16 becomes `[1, 2, 3, 15]`.
//
// # Panic
// This function panics if any character is not a valid number for the given base
fn to_radix(number_with_leading_zeros: &str, base: u32) -> Vec<u32> {
    let number = number_with_leading_zeros.trim_start_matches('0');
    let mut digits = Vec::with_capacity(number.len());
    for c in number.chars().filter(|&c| c != '_') {
        digits.push(c.to_digit(base).expect("Digit invalid for given base"));
    }
    digits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal() {
        assert_eq!(bignum!(123), bignum!(123));
        assert_eq!(bignum!(123), bignum!(0123));
        assert_ne!(bignum!(123), bignum!(321));
    }

    #[test]
    fn test_different_radix() {
        let base_10 = bignum!(234793475345234234);
        let base_16 = bignum!(0x342276BFD88393A);
        let base_8 = bignum!(0o15021166577542034472);
        let base_2 = bignum!(0b1101000010001001110110101111111101100010000011100100111010);

        assert_eq!(base_10, base_16);
        assert_eq!(base_16, base_8);
        assert_eq!(base_8, base_2);
    }

    #[test]
    fn test_add() {
        let a = bignum!(45600000000000000000000000000000000000000999);
        let b = bignum!(12300000000000000000000000000000000000000456);
        let d = bignum!(57900000000000000000000000000000000000001455);

        assert_eq!(a + b, d);
    }

    #[test]
    fn test_shl() {
        assert_eq!(
            &bignum!(1) << 128,
            bignum!(0x100000000000000000000000000000000)
        );

        assert_eq!(
            &bignum!(0xdeadbeef) << 63,
            bignum!(0x6f56df778000000000000000),
        )
    }

    #[test]
    fn test_from_be_bytes() {
        assert_eq!(
            BigNum::from_be_bytes(&[0xde, 0xad, 0xbe, 0xef]),
            bignum!(0xdeadbeef)
        )
    }
}
