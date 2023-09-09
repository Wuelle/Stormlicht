use sl_std::ascii;

/// <https://infra.spec.whatwg.org/#c0-control>
#[inline]
pub(crate) fn is_c0_control(c: u8) -> bool {
    c <= 0x1F
}

/// <https://url.spec.whatwg.org/#c0-control-percent-encode-set>
pub(crate) fn is_c0_percent_encode_set(c: u8) -> bool {
    is_c0_control(c) | matches!(c, 0x7F..)
}

/// <https://url.spec.whatwg.org/#fragment-percent-encode-set>
pub(crate) fn is_fragment_percent_encode_set(c: u8) -> bool {
    is_c0_percent_encode_set(c) | matches!(c, b' ' | b'"' | b'#' | b'<' | b'>')
}

/// <https://url.spec.whatwg.org/#query-percent-encode-set>
pub(crate) fn is_query_percent_encode_set(c: u8) -> bool {
    is_c0_percent_encode_set(c) | matches!(c, b' ' | b'"' | b'#' | b'<' | b'>')
}

/// <https://url.spec.whatwg.org/#special-query-percent-encode-set>
pub(crate) fn is_special_query_percent_encode_set(c: u8) -> bool {
    is_query_percent_encode_set(c) || c == b'\''
}

/// <https://url.spec.whatwg.org/#path-percent-encode-set>
pub(crate) fn is_path_percent_encode_set(c: u8) -> bool {
    is_query_percent_encode_set(c) | matches!(c, b'?' | b'`' | b'{' | b'}')
}

/// <https://url.spec.whatwg.org/#userinfo-percent-encode-set>
pub(crate) fn is_userinfo_percent_encode_set(c: u8) -> bool {
    is_path_percent_encode_set(c)
        | matches!(c, b'/' | b':' | b';' | b'=' | b'@' | b'['..=b'^' | b'|')
}
/// <https://url.spec.whatwg.org/#string-percent-encode-after-encoding>
pub fn percent_encode<W: ascii::Write, F: Fn(u8) -> bool>(
    input: &str,
    in_encode_set: F,
    writer: &mut W,
) {
    for c in input.chars() {
        percent_encode_char(c, &in_encode_set, writer);
    }
}

#[inline]
pub fn percent_encode_char<W: ascii::Write, F: Fn(u8) -> bool>(
    c: char,
    in_encode_set: F,
    writer: &mut W,
) {
    let mut buffer = [0; 4];
    c.encode_utf8(&mut buffer);
    for &b in buffer.iter().take(c.len_utf8()) {
        if let Some(c) = ascii::Char::from_u8(b) && !in_encode_set(b) {
            writer.write_char(c)
        } else {
            percent_encode_byte(b, writer);
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

#[cfg(test)]
mod tests {
    use sl_std::ascii;

    use super::percent_encode_byte;

    #[test]
    fn test_percent_encode_byte() {
        // Examples from
        // <https://url.spec.whatwg.org/#example-percent-encode-operations>

        let mut buffer = ascii::String::default();
        percent_encode_byte(0x23, &mut buffer);
        assert_eq!(buffer.as_str(), "%23");

        buffer.clear();
        percent_encode_byte(0x7F, &mut buffer);
        assert_eq!(buffer.as_str(), "%7F");
    }
}