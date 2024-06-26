use sl_std::ascii;

use crate::AsciiSet;

/// <https://infra.spec.whatwg.org/#c0-control>
pub const C0_CONTROL: AsciiSet =
    AsciiSet::from_range_inclusive(ascii::Char::Null, ascii::Char::InformationSeparatorOne);

#[inline]
#[must_use]
pub(crate) fn is_c0_control(c: u8) -> bool {
    c <= 0x1F
}

/// <https://url.spec.whatwg.org/#c0-control-percent-encode-set>
#[inline]
#[must_use]
pub(crate) fn is_c0_percent_encode_set(c: u8) -> bool {
    is_c0_control(c) | matches!(c, 0x7F..)
}

/// <https://url.spec.whatwg.org/#fragment-percent-encode-set>
#[inline]
#[must_use]
pub(crate) fn is_fragment_percent_encode_set(c: u8) -> bool {
    is_c0_percent_encode_set(c) | matches!(c, b' ' | b'"' | b'#' | b'<' | b'>')
}

/// <https://url.spec.whatwg.org/#query-percent-encode-set>
#[inline]
#[must_use]
pub(crate) fn is_query_percent_encode_set(c: u8) -> bool {
    is_c0_percent_encode_set(c) | matches!(c, b' ' | b'"' | b'#' | b'<' | b'>')
}

/// <https://url.spec.whatwg.org/#special-query-percent-encode-set>
#[inline]
#[must_use]
pub(crate) fn is_special_query_percent_encode_set(c: u8) -> bool {
    is_query_percent_encode_set(c) || c == b'\''
}

/// <https://url.spec.whatwg.org/#path-percent-encode-set>
#[inline]
#[must_use]
pub(crate) fn is_path_percent_encode_set(c: u8) -> bool {
    is_query_percent_encode_set(c) | matches!(c, b'?' | b'`' | b'{' | b'}')
}

/// <https://url.spec.whatwg.org/#userinfo-percent-encode-set>
#[inline]
#[must_use]
pub(crate) fn is_userinfo_percent_encode_set(c: u8) -> bool {
    is_path_percent_encode_set(c)
        | matches!(c, b'/' | b':' | b';' | b'=' | b'@' | b'['..=b'^' | b'|')
}
/// <https://url.spec.whatwg.org/#string-percent-encode-after-encoding>
pub fn percent_encode<W: ascii::Write, F: Fn(u8) -> bool>(
    input: &[u8],
    in_encode_set: F,
    writer: &mut W,
) {
    for &byte in input {
        if let Some(c) = ascii::Char::from_u8(byte)
            && !in_encode_set(byte)
        {
            writer.write_char(c)
        } else {
            percent_encode_byte(byte, writer);
        }
    }
}

/// <https://url.spec.whatwg.org/#percent-encode>
#[inline]
fn percent_encode_byte<W: ascii::Write>(byte: u8, writer: &mut W) {
    const HEX_DIGITS: [ascii::Char; 16] = [
        ascii::Char::Digit0,
        ascii::Char::Digit1,
        ascii::Char::Digit2,
        ascii::Char::Digit3,
        ascii::Char::Digit4,
        ascii::Char::Digit5,
        ascii::Char::Digit6,
        ascii::Char::Digit7,
        ascii::Char::Digit8,
        ascii::Char::Digit9,
        ascii::Char::CapitalA,
        ascii::Char::CapitalB,
        ascii::Char::CapitalC,
        ascii::Char::CapitalD,
        ascii::Char::CapitalE,
        ascii::Char::CapitalF,
    ];

    let chars = &[
        ascii::Char::PercentSign,
        HEX_DIGITS[(byte / 16) as usize],
        HEX_DIGITS[(byte % 16) as usize],
    ];
    writer.write_str(ascii::Str::from_ascii_chars(chars));
}

/// <https://url.spec.whatwg.org/#percent-decode>
#[must_use]
pub fn percent_decode(encoded: &ascii::Str) -> Vec<u8> {
    let decode = |first: ascii::Char, second: ascii::Char| {
        let value = first.to_char().to_digit(16)? * 16 + second.to_char().to_digit(16)?;

        // Truncating to a u8 is safe here because we only read two hex digits -> 0xFF max
        Some(value as u8)
    };

    // 1. Let output be an empty byte sequence.
    let mut result = Vec::with_capacity(encoded.len());

    // 2. For each byte byte in input:
    let chars = encoded.chars();
    let mut i = 0;
    while i < chars.len() {
        // 1. If byte is not 0x25 (%), then append byte to output.
        if chars[i] != ascii::Char::PercentSign {
            result.push(chars[i].to_u8());
        } else if i + 2 < chars.len()
            && let Some(c) = decode(chars[i + 1], chars[i + 2])
        {
            result.push(c);
            i += 2;
        } else {
            result.push(chars[i].to_u8());
        }
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use sl_std::ascii;

    use super::{percent_decode, percent_encode_byte};

    #[test]
    fn test_percent_encode_byte() {
        // Examples from
        // https://url.spec.whatwg.org/#example-percent-encode-operations

        let mut buffer = ascii::String::default();
        percent_encode_byte(0x23, &mut buffer);
        assert_eq!(buffer.as_str(), "%23");

        buffer.clear();
        percent_encode_byte(0x7F, &mut buffer);
        assert_eq!(buffer.as_str(), "%7F");
    }

    #[test]
    fn test_percent_decode() {
        // Examples from
        // https://url.spec.whatwg.org/#example-percent-encode-operations
        let encoded = "%25%s%1G".try_into().unwrap();
        let decoded = percent_decode(encoded);
        assert_eq!(decoded, b"%%s%1G");
    }
}
