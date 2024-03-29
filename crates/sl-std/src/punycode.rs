//! Punycode implementation as per <https://www.rfc-editor.org/rfc/rfc3492>
//!
//! Also [IDNA](https://de.wikipedia.org/wiki/Internationalizing_Domain_Names_in_Applications)

use crate::ascii;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PunyCodeError {
    /// Integer overflows are explicitly forbidden in punycode
    IntegerOverflow,
    /// Trying to decode invalid (i.e non-ascii) punycode
    InvalidPunycode,
    InvalidCharacterCode,
}

const BASE: u32 = 36;
const TMIN: u32 = 1;
const TMAX: u32 = 26;
const SKEW: u32 = 38;
const DAMP: u32 = 700;
const INITIAL_BIAS: u32 = 72;
const INITIAL_N: u32 = 128;
const DIGITS: [ascii::Char; BASE as usize] = [
    ascii::Char::SmallA,
    ascii::Char::SmallB,
    ascii::Char::SmallC,
    ascii::Char::SmallD,
    ascii::Char::SmallE,
    ascii::Char::SmallF,
    ascii::Char::SmallG,
    ascii::Char::SmallH,
    ascii::Char::SmallI,
    ascii::Char::SmallJ,
    ascii::Char::SmallK,
    ascii::Char::SmallL,
    ascii::Char::SmallM,
    ascii::Char::SmallN,
    ascii::Char::SmallO,
    ascii::Char::SmallP,
    ascii::Char::SmallQ,
    ascii::Char::SmallR,
    ascii::Char::SmallS,
    ascii::Char::SmallT,
    ascii::Char::SmallU,
    ascii::Char::SmallV,
    ascii::Char::SmallW,
    ascii::Char::SmallX,
    ascii::Char::SmallY,
    ascii::Char::SmallZ,
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
];

#[must_use]
fn adapt(mut delta: u32, num_points: u32, is_first: bool) -> u32 {
    delta /= if is_first { DAMP } else { 2 };

    delta += delta / num_points;
    let mut k = 0;

    while delta > ((BASE - TMIN) * TMAX) / 2 {
        delta /= BASE - TMIN;
        k += BASE;
    }

    k + (((BASE - TMIN + 1) * delta) / (delta + SKEW))
}

pub fn punycode_encode(input: &str) -> Result<ascii::String, PunyCodeError> {
    let mut n = INITIAL_N;
    let mut delta: u32 = 0;
    let mut bias = INITIAL_BIAS;
    let num_basic = input.chars().filter(|c| c.is_ascii()).count() as u32;
    let mut h = num_basic;

    let mut output: ascii::String = input.chars().filter_map(|c| c.as_ascii()).collect();
    let input_len = input.chars().count() as u32;
    if num_basic > 0 && num_basic != input_len {
        output.push(ascii::Char::HyphenMinus);
    }
    while h < input_len {
        let m = input.chars().filter(|c| *c as u32 >= n).min().unwrap() as u32;
        delta = delta
            .checked_add(
                (m - n)
                    .checked_mul(h + 1)
                    .ok_or(PunyCodeError::IntegerOverflow)?,
            )
            .ok_or(PunyCodeError::IntegerOverflow)?;
        n = m;

        for c in input.chars().map(|c| c as u32) {
            if c < n {
                delta += 1;
            }

            if c == n {
                let mut q = delta;

                let mut k = BASE;
                loop {
                    let threshold = if k <= bias + TMIN {
                        TMIN
                    } else if k >= bias + TMAX {
                        TMAX
                    } else {
                        k - bias
                    };

                    if q < threshold {
                        break;
                    }
                    let codepoint_numeric = threshold + ((q - threshold) % (BASE - threshold));
                    output.push(DIGITS[codepoint_numeric as usize]);

                    q = (q - threshold) / (BASE - threshold);
                    k += BASE;
                }

                output.push(DIGITS[q as usize]);
                bias = adapt(delta, h + 1, h == num_basic);
                delta = 0;
                h += 1;
            }
        }
        delta += 1;
        n += 1;
    }
    Ok(output)
}

pub fn punycode_decode(input: &ascii::Str) -> Result<String, PunyCodeError> {
    let (mut output, extended) = match input.rfind(ascii::Char::HyphenMinus) {
        Some(i) => {
            if i != input.len() - 1 {
                (input[..i].as_str().chars().collect(), &input[i + 1..])
            } else {
                // If there are no trailing special characters, the dash was not a seperator,
                // it was part of the literal ascii str
                (input[..i + 1].as_str().chars().collect(), &input[i + 1..])
            }
        },
        None => (vec![], input),
    };

    let mut n = INITIAL_N;
    let mut i: u32 = 0;
    let mut bias = INITIAL_BIAS;

    let mut codepoints = extended.chars().iter().peekable();
    while codepoints.peek().is_some() {
        let old_i = i;
        let mut weight = 1;
        let mut k = BASE;
        loop {
            let code_point = codepoints.next().ok_or(PunyCodeError::IntegerOverflow)?;
            let digit = DIGITS
                .iter()
                .position(|d| d == code_point)
                .ok_or(PunyCodeError::InvalidPunycode)? as u32;

            i = i
                .checked_add(
                    digit
                        .checked_mul(weight)
                        .ok_or(PunyCodeError::IntegerOverflow)?,
                )
                .ok_or(PunyCodeError::IntegerOverflow)?;

            let threshold = if k <= bias + TMIN {
                TMIN
            } else if k >= bias + TMAX {
                TMAX
            } else {
                k - bias
            };

            if digit < threshold {
                break;
            }

            weight = weight
                .checked_mul(BASE - threshold)
                .ok_or(PunyCodeError::IntegerOverflow)?;
            k += BASE;
        }

        let num_points = output.len() as u32 + 1;
        bias = adapt(i - old_i, num_points, old_i == 0);
        n = n
            .checked_add(i / num_points)
            .ok_or(PunyCodeError::IntegerOverflow)?;
        i %= num_points;

        output.insert(
            i as usize,
            char::try_from(n).map_err(|_| PunyCodeError::InvalidCharacterCode)?,
        );
        i += 1;
    }
    Ok(output.iter().collect())
}

/// The returned value is guaranteed to be pure ascii
pub fn idna_encode(input: &str) -> Result<String, PunyCodeError> {
    // Don't encode strings that are already pure ascii
    if input.is_ascii() {
        Ok(input.to_string())
    } else {
        Ok(format!("xn--{}", punycode_encode(input)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // https://www.rfc-editor.org/rfc/rfc3492#section-7.1
    const ARABIC: & str = "\u{0644}\u{064A}\u{0647}\u{0645}\u{0627}\u{0628}\u{062A}\u{0643}\u{0644}\u{0645}\u{0648}\u{0634}\u{0639}\u{0631}\u{0628}\u{064A}\u{061F}";
    const ARABIC_ENCODED: &ascii::Str = ascii::Str::from_bytes(b"egbpdaj6bu4bxfgehfvwxn").unwrap();

    const CHINESE: &str =
        "\u{4ED6}\u{4EEC}\u{4E3A}\u{4EC0}\u{4E48}\u{4E0D}\u{8BF4}\u{4E2D}\u{6587}";
    const CHINESE_ENCODED: &ascii::Str =
        ascii::Str::from_bytes(b"ihqwcrb4cv8a8dqg056pqjye").unwrap();

    const CHINESE_2: &str =
        "\u{4ED6}\u{5011}\u{7232}\u{4EC0}\u{9EBD}\u{4E0D}\u{8AAA}\u{4E2D}\u{6587}";
    const CHINESE_ENCODED_2: &ascii::Str =
        ascii::Str::from_bytes(b"ihqwctvzc91f659drss3x8bo0yb").unwrap();

    const CZECH: &str = "\u{0050}\u{0072}\u{006F}\u{010D}\u{0070}\u{0072}\u{006F}\u{0073}\u{0074}\u{011B}\u{006E}\u{0065}\u{006D}\u{006C}\u{0075}\u{0076}\u{00ED}\u{010D}\u{0065}\u{0073}\u{006B}\u{0079}";
    const CZECH_ENCODED: &ascii::Str =
        ascii::Str::from_bytes(b"Proprostnemluvesky-uyb24dma41a").unwrap();

    const HEBREW: &str = "\u{05DC}\u{05DE}\u{05D4}\u{05D4}\u{05DD}\u{05E4}\u{05E9}\u{05D5}\u{05D8}\u{05DC}\u{05D0}\u{05DE}\u{05D3}\u{05D1}\u{05E8}\u{05D9}\u{05DD}\u{05E2}\u{05D1}\u{05E8}\u{05D9}\u{05EA}";
    const HEBREW_ENCODED: &ascii::Str =
        ascii::Str::from_bytes(b"4dbcagdahymbxekheh6e0a7fei0b").unwrap();

    const HINDI: &str = "\u{092F}\u{0939}\u{0932}\u{094B}\u{0917}\u{0939}\u{093F}\u{0928}\u{094D}\u{0926}\u{0940}\u{0915}\u{094D}\u{092F}\u{094B}\u{0902}\u{0928}\u{0939}\u{0940}\u{0902}\u{092C}\u{094B}\u{0932}\u{0938}\u{0915}\u{0924}\u{0947}\u{0939}\u{0948}\u{0902}";
    const HINDI_ENCODED: &ascii::Str =
        ascii::Str::from_bytes(b"i1baa7eci9glrd9b2ae1bj0hfcgg6iyaf8o0a1dig0cd").unwrap();

    const JAPANESE: &str = "\u{306A}\u{305C}\u{307F}\u{3093}\u{306A}\u{65E5}\u{672C}\u{8A9E}\u{3092}\u{8A71}\u{3057}\u{3066}\u{304F}\u{308C}\u{306A}\u{3044}\u{306E}\u{304B}";
    const JAPANESE_ENCODED: &ascii::Str =
        ascii::Str::from_bytes(b"n8jok5ay5dzabd5bym9f0cm5685rrjetr6pdxa").unwrap();

    const KOREAN: &str = "\u{C138}\u{ACC4}\u{C758}\u{BAA8}\u{B4E0}\u{C0AC}\u{B78C}\u{B4E4}\u{C774}\u{D55C}\u{AD6D}\u{C5B4}\u{B97C}\u{C774}\u{D574}\u{D55C}\u{B2E4}\u{BA74}\u{C5BC}\u{B9C8}\u{B098}\u{C88B}\u{C744}\u{AE4C}";
    const KOREAN_ENCODED: &ascii::Str = ascii::Str::from_bytes(
        b"989aomsvi5e83db1d2a355cv1e0vak1dwrv93d5xbh15a0dt30a5jpsd879ccm6fea98c",
    )
    .unwrap();

    // NOTE: this spec version of test includes an uppercase "D"
    // We don't support capitalized letters in the non-basic text (at least not during encoding)
    // Therefore, i replaced it with a lowercase "d"
    const RUSSIAN: & str = "\u{043F}\u{043E}\u{0447}\u{0435}\u{043C}\u{0443}\u{0436}\u{0435}\u{043E}\u{043D}\u{0438}\u{043D}\u{0435}\u{0433}\u{043E}\u{0432}\u{043E}\u{0440}\u{044F}\u{0442}\u{043F}\u{043E}\u{0440}\u{0443}\u{0441}\u{0441}\u{043A}\u{0438}";
    const RUSSIAN_ENCODED: &ascii::Str =
        ascii::Str::from_bytes(b"b1abfaaepdrnnbgefbadotcwatmq2g4l").unwrap();

    const SPANISH: &str = "Porqu\u{00E9}nopuedensimplementehablarenEspa\u{00F1}ol";
    const SPANISH_ENCODED: &ascii::Str =
        ascii::Str::from_bytes(b"PorqunopuedensimplementehablarenEspaol-fmd56a").unwrap();

    const VIETNAMESE: &str =
        "T\u{1EA1}isaoh\u{1ECD}kh\u{00F4}ngth\u{1EC3}ch\u{1EC9}n\u{00F3}iti\u{1EBF}ngVi\u{1EC7}t";
    const VIETNAMESE_ENCODED: &ascii::Str =
        ascii::Str::from_bytes(b"TisaohkhngthchnitingVit-kjcr8268qyxafd2f1b9g").unwrap();

    const JAPANESE_2: &str = "3\u{5E74}B\u{7D44}\u{91D1}\u{516B}\u{5148}\u{751F}";
    const JAPANESE_ENCODED_2: &ascii::Str =
        ascii::Str::from_bytes(b"3B-ww4c5e180e575a65lsy2b").unwrap();

    const JAPANESE_3: &str = "\u{5B89}\u{5BA4}\u{5948}\u{7F8E}\u{6075}-with-SUPER-MONKEYS";
    const JAPANESE_ENCODED_3: &ascii::Str =
        ascii::Str::from_bytes(b"-with-SUPER-MONKEYS-pc58ag80a8qai00g7n9n").unwrap();

    const JAPANESE_4: &str =
        "Hello-Another-Way-\u{305D}\u{308C}\u{305E}\u{308C}\u{306E}\u{5834}\u{6240}";
    const JAPANESE_ENCODED_4: &ascii::Str =
        ascii::Str::from_bytes(b"Hello-Another-Way--fc4qua05auwb3674vfr0b").unwrap();

    const JAPANESE_5: &str = "\u{3072}\u{3068}\u{3064}\u{5C4B}\u{6839}\u{306E}\u{4E0B}2";
    const JAPANESE_ENCODED_5: &ascii::Str = ascii::Str::from_bytes(b"2-u9tlzr9756bt3uc0v").unwrap();

    const JAPANESE_6: &str = "Maji\u{3067}Koi\u{3059}\u{308B}5\u{79D2}\u{524D}";
    const JAPANESE_ENCODED_6: &ascii::Str =
        ascii::Str::from_bytes(b"MajiKoi5-783gue6qz075azm5e").unwrap();

    const JAPANESE_7: &str = "\u{30D1}\u{30D5}\u{30A3}\u{30FC}de\u{30EB}\u{30F3}\u{30D0}";
    const JAPANESE_ENCODED_7: &ascii::Str = ascii::Str::from_bytes(b"de-jg4avhby1noc0d").unwrap();

    const JAPANESE_8: &str = "\u{305D}\u{306E}\u{30B9}\u{30D4}\u{30FC}\u{30C9}\u{3067}";
    const JAPANESE_ENCODED_8: &ascii::Str = ascii::Str::from_bytes(b"d9juau41awczczp").unwrap();

    const PURE_ASCII: &str = "-> $1.00 <-";
    const PURE_ASCII_ENCODED: &ascii::Str = ascii::Str::from_bytes(b"-> $1.00 <-").unwrap();

    #[test]
    fn test_punycode_decode() {
        assert_eq!(punycode_decode(ARABIC_ENCODED).unwrap(), ARABIC);
        assert_eq!(punycode_decode(CHINESE_ENCODED).unwrap(), CHINESE);
        assert_eq!(punycode_decode(CHINESE_ENCODED_2).unwrap(), CHINESE_2);
        assert_eq!(punycode_decode(CZECH_ENCODED).unwrap(), CZECH);
        assert_eq!(punycode_decode(HEBREW_ENCODED).unwrap(), HEBREW);
        assert_eq!(punycode_decode(HINDI_ENCODED).unwrap(), HINDI);
        assert_eq!(punycode_decode(JAPANESE_ENCODED).unwrap(), JAPANESE);
        assert_eq!(punycode_decode(KOREAN_ENCODED).unwrap(), KOREAN);
        assert_eq!(punycode_decode(RUSSIAN_ENCODED).unwrap(), RUSSIAN);
        assert_eq!(punycode_decode(SPANISH_ENCODED).unwrap(), SPANISH);
        assert_eq!(punycode_decode(VIETNAMESE_ENCODED).unwrap(), VIETNAMESE);
        assert_eq!(punycode_decode(JAPANESE_ENCODED_2).unwrap(), JAPANESE_2);
        assert_eq!(punycode_decode(JAPANESE_ENCODED_3).unwrap(), JAPANESE_3);
        assert_eq!(punycode_decode(JAPANESE_ENCODED_4).unwrap(), JAPANESE_4);
        assert_eq!(punycode_decode(JAPANESE_ENCODED_5).unwrap(), JAPANESE_5);
        assert_eq!(punycode_decode(JAPANESE_ENCODED_6).unwrap(), JAPANESE_6);
        assert_eq!(punycode_decode(JAPANESE_ENCODED_7).unwrap(), JAPANESE_7);
        assert_eq!(punycode_decode(JAPANESE_ENCODED_8).unwrap(), JAPANESE_8);
        assert_eq!(punycode_decode(PURE_ASCII_ENCODED).unwrap(), PURE_ASCII);
    }

    #[test]
    fn test_punycode_encode() {
        use std::ops::Deref;

        assert_eq!(punycode_encode(ARABIC).unwrap().deref(), ARABIC_ENCODED);
        assert_eq!(punycode_encode(CHINESE).unwrap().deref(), CHINESE_ENCODED);
        assert_eq!(
            punycode_encode(CHINESE_2).unwrap().deref(),
            CHINESE_ENCODED_2
        );
        assert_eq!(punycode_encode(CZECH).unwrap().deref(), CZECH_ENCODED);
        assert_eq!(punycode_encode(HEBREW).unwrap().deref(), HEBREW_ENCODED);
        assert_eq!(punycode_encode(HINDI).unwrap().deref(), HINDI_ENCODED);
        assert_eq!(punycode_encode(JAPANESE).unwrap().deref(), JAPANESE_ENCODED);
        assert_eq!(punycode_encode(KOREAN).unwrap().deref(), KOREAN_ENCODED);
        assert_eq!(punycode_encode(RUSSIAN).unwrap().deref(), RUSSIAN_ENCODED);
        assert_eq!(punycode_encode(SPANISH).unwrap().deref(), SPANISH_ENCODED);
        assert_eq!(
            punycode_encode(VIETNAMESE).unwrap().deref(),
            VIETNAMESE_ENCODED
        );
        assert_eq!(
            punycode_encode(JAPANESE_2).unwrap().deref(),
            JAPANESE_ENCODED_2
        );
        assert_eq!(
            punycode_encode(JAPANESE_3).unwrap().deref(),
            JAPANESE_ENCODED_3
        );
        assert_eq!(
            punycode_encode(JAPANESE_4).unwrap().deref(),
            JAPANESE_ENCODED_4
        );
        assert_eq!(
            punycode_encode(JAPANESE_5).unwrap().deref(),
            JAPANESE_ENCODED_5
        );
        assert_eq!(
            punycode_encode(JAPANESE_6).unwrap().deref(),
            JAPANESE_ENCODED_6
        );
        assert_eq!(
            punycode_encode(JAPANESE_7).unwrap().deref(),
            JAPANESE_ENCODED_7
        );
        assert_eq!(
            punycode_encode(JAPANESE_8).unwrap().deref(),
            JAPANESE_ENCODED_8
        );
        assert_eq!(
            punycode_encode(PURE_ASCII).unwrap().deref(),
            PURE_ASCII_ENCODED
        );
    }
}
