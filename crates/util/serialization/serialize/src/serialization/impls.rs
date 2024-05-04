use std::{ascii, collections::HashMap};

use super::{Serialize, SerializeMap, SerializeSequence, Serializer};

impl<'a> Serialize for &'a str {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_string(self)
    }
}

impl Serialize for String {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_string(self.as_str())
    }
}

impl Serialize for usize {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_usize(*self)
    }
}

impl Serialize for u8 {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_usize(*self as usize)
    }
}

impl Serialize for u16 {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_usize(*self as usize)
    }
}

impl Serialize for u32 {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_usize(*self as usize)
    }
}

impl Serialize for u64 {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_usize(*self as usize)
    }
}

impl Serialize for u128 {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_usize(*self as usize)
    }
}

impl<T> Serialize for [T]
where
    T: Serialize,
{
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_sequence()?;

        for element in self {
            sequence.serialize_element(element)?;
        }
        sequence.finish()?;

        Ok(())
    }
}

impl<T> Serialize for Vec<T>
where
    T: Serialize,
{
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        self.as_slice().serialize_to(serializer)
    }
}

impl<'a, K, V> Serialize for HashMap<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map()?;

        for (key, value) in self {
            map.serialize_key_value_pair(key, value)?;
        }
        map.finish()?;

        Ok(())
    }
}

impl Serialize for ascii::Char {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        (*self as u8).serialize_to(serializer)
    }
}

impl<T> Serialize for Option<T>
where
    T: Serialize,
{
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_option(self)
    }
}
