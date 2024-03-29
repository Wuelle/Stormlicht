mod autogenerated {
    include!(concat!(env!("OUT_DIR"), "/object_identifier.rs"));
}
pub use autogenerated::{ObjectIdentifier, UnknownObjectIdentifier};

use super::{Error, Primitive, TypeTag};

impl<'a> Primitive<'a> for ObjectIdentifier {
    type Error = Error;

    const TYPE_TAG: TypeTag = TypeTag::OBJECT_IDENTIFIER;

    fn from_value_bytes(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        if bytes.is_empty() {
            return Err(Self::Error::UnexpectedEOF);
        }

        let v1 = bytes[0] / 40;
        let v2 = bytes[0] % 40;

        let is_valid = matches!((v1, v2), (0 | 1, ..=39) | (2, _));
        if !is_valid {
            return Err(Self::Error::IllegalValue);
        }

        let mut digits = vec![v1 as usize, v2 as usize];
        let mut remaining_bytes = bytes[1..].iter();
        while let Some(byte) = remaining_bytes.next() {
            let mut has_more = byte >> 7 != 0;
            let mut value = (byte & 0b01111111) as usize;

            while has_more {
                let byte = remaining_bytes.next().ok_or(super::Error::UnexpectedEOF)?;
                value <<= 7;
                value |= (byte & 0b01111111) as usize;
                has_more = byte >> 7 != 0;
            }

            digits.push(value);
        }

        Ok(Self::try_from_digits(&digits)?)
    }
}
