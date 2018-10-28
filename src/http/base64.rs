///
/// [0 - 25] map `A` to `Z` (ascii 65 to 90)
/// [26 - 51] map `a` to `z` (ascii 97 to 122)
/// [52 - 61] map `0` to `9` (ascii 48 to 57)
/// [62] map `+` (ascii 43) or `-` (ascii 45)
/// [63] map `/` (ascii 47) or `_` (ascii 95)
/// padding `=` (ascii 61)
///
const STANDARD_INDEX: [u8;64] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z',
    b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z',
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
    b'+',
    b'/'
];
const PADDING: u8 = 61;

#[derive(Debug)]
pub struct DecodeError {
    cause: Cause,
    desc: String,
}

#[derive(Debug, PartialEq)]
pub enum Cause {
    IncorrectPadding,
    IncorrectCharacter,
}

///
/// Reverse index calculation
/// given a character c, to get the index value:
/// c is uppercase => c - 65
/// c is lowercase => c - 71
/// c is number => c + 4
/// c is `+` => c + 19, c is `-` => c + 17
/// c is `/` => c + 16, c is `_` => c - 32
///
fn get_index_by_character(c: u8) -> Result<u8, DecodeError> {
    if c.is_ascii_uppercase() { return Ok(c - 65); }
    if c.is_ascii_lowercase() { return Ok(c - 71); }
    if c.is_ascii_digit() { return Ok(c + 4); }
    if c == b'+' { return Ok(c + 19); }
    if c == b'-' { return Ok(c + 17); }
    if c == b'/' { return Ok(c + 16); }
    if c == b'_' { return Ok(c - 32); }
    if c == b'=' { return Ok(0); }
    Err(DecodeError {
        cause: Cause::IncorrectCharacter,
        desc: String::from(format!("character {} is invalid base64 character", char::from(c)))
    })
}

///
/// test if the character is the valid base64 character
///
fn is_base64_char(c: u8) -> bool {
    match c {
        43 | 45 | 47 | 95 => true,
        upper if upper >= 65 && upper <= 90 => true,
        lower if lower >= 97 && lower <= 122 => true,
        digit if digit >= 48 && digit <= 57 => true,
        _ => false
    }
}

///
/// Encode string s using the standard Base64 alphabet
///
pub fn base64_encode(s: &str) -> String {

    let mut buffer = [0u8;3];

    let bytes = s.as_bytes();
    let len = s.len();

    let loop_num = len / 3;
    let remaining = len % 3;

    let mut res = String::new();

    for i in 0..(loop_num + 1) {
        if i == loop_num {
            if remaining == 1 {
                buffer[0] = bytes[len - 1];
                buffer[1] = 0;
                buffer[2] = 0;
            }
            else if remaining == 2 {
                buffer[0] = bytes[len - 2];
                buffer[1] = bytes[len - 1];
                buffer[2] = 0;
            }
            else { break; }
        } else {
            buffer[0] = bytes[i * 3];
            buffer[1] = bytes[i * 3 + 1];
            buffer[2] = bytes[i * 3 + 2];
        }

        // precedence: `+` > shift > '&'
        let index0 = (buffer[0] >> 2) as usize;
        let index1 = ((buffer[0] << 4 & 0b11_1111) + (buffer[1] >> 4)) as usize;
        let index2 = (((buffer[1] & 0b00_1111) << 2) + (buffer[2] >> 6)) as usize;
        let index3 = (buffer[2] & 0b11_1111) as usize;

        res.push(char::from(STANDARD_INDEX[index0]));
        res.push(char::from(STANDARD_INDEX[index1]));

        if i == loop_num {
            if remaining == 1 {
                res.push(char::from(b'='));
                res.push(char::from(b'='));
            }
            else if remaining == 2 {
                res.push(char::from(STANDARD_INDEX[index2]));
                res.push(char::from(b'='));
            }
        }
        else {
            res.push(char::from(STANDARD_INDEX[index2]));
            res.push(char::from(STANDARD_INDEX[index3]));
        }
    }

    res
}

///
/// Decode string s using the standard Base64 alphabet.
///
pub fn base64_decode(s: &str) -> Result<String, DecodeError> {

    let bytes = s.as_bytes();
    let len = s.len();

    // deal with corner case first
    if s.is_empty() {
        return Ok("".to_string());
    }

    if len % 4 != 0 {
        return Err(DecodeError {
            cause: Cause::IncorrectPadding,
            desc: String::from("length of base64 string should be divisible by 4"),
        });
    }
    else {
        let prefix = &s[0..len - 2];
        for c in prefix.bytes() {
            if !is_base64_char(c) {
                return Err(DecodeError {
                    cause: Cause::IncorrectPadding,
                    desc: String::from("string contains invalid character"),
                });
            }
        }

        let padding1 = bytes[len - 2];
        let padding2 = bytes[len - 1];

        if !is_base64_char(padding1) && padding1 != b'=' || !is_base64_char(padding2) && padding2 != b'=' {
            return Err(DecodeError {
                cause: Cause::IncorrectPadding,
                desc: String::from("padding character must be '='"),
            });
        }

        if padding1 == b'=' && padding2 != b'=' {
            return Err(DecodeError {
                cause: Cause::IncorrectPadding,
                desc: String::from("padding must be either '=' or '=='"),
            });
        }
    }

    let mut buffer = [0u8;3];


    let loop_num = len / 4;

    let mut res = String::new();

    for i in 0..loop_num {
        let c0 = bytes[i * 4];
        let c1 = bytes[i * 4 + 1];
        let c2 = bytes[i * 4 + 2];
        let c3 = bytes[i * 4 + 3];

        let index0 = get_index_by_character(c0)?;
        let index1 = get_index_by_character(c1)?;
        let index2 = get_index_by_character(c2)?;
        let index3 = get_index_by_character(c3)?;

        buffer[0] = (index0 << 2) + (index1 >> 4);
        buffer[1] = ((index1 & 0b00_1111) << 4) + (index2 >> 2);
        buffer[2] = ((index2 & 0b00_0011) << 6) + index3;

        res.push(char::from(buffer[0]));
        if buffer[1] != 0 {
            res.push(char::from(buffer[1]));
        }
        if buffer[2] != 0 {
            res.push(char::from(buffer[2]));
        }
    }

    Ok(res)
}

///
/// Encode string s using the URL- and filesystem-safe alphabet,
/// which substitutes `-` instead of `+` and `_` instead of `/` in the standard Base64 alphabet.
/// The result can still contain =.
///
pub fn base64_encode_urlsafe(s: &str) -> String {
    // TODO
    unimplemented!()
}

///
/// Decode string s using the URL- and filesystem-safe alphabet,
/// which substitutes `-` instead of `+` and `_` instead of `/` in the standard Base64 alphabet.
///
pub fn base64_decode_urlsafe(s: &str) -> String {
    // TODO
    unimplemented!()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_standard_encode() {
        let input = "";
        let output = base64_encode(input);
        assert_eq!(output.as_str(), "");

        let input = "Man";
        let output = base64_encode(input);
        assert_eq!(output.as_str(), "TWFu");

        let input = "A";
        let output = base64_encode(input);
        assert_eq!(output.as_str(), "QQ==");

        let input = "BC";
        let output = base64_encode(input);
        assert_eq!(output.as_str(), "QkM=");

        let input = "e.";
        let output = base64_encode(input);
        assert_eq!(output.as_str(), "ZS4=");

        let input = "严";
        let output = base64_encode(input);
        assert_eq!(output.as_str(), "5Lil");

        let input = "\
        Man is distinguished, not only by his reason, \
        but by this singular passion from other animals, \
        which is a lust of the mind, that by a perseverance \
        of delight in the continued and indefatigable generation \
        of knowledge, exceeds the short vehemence of any carnal pleasure.";
        let output = base64_encode(input);
        let expected = "TWFuIGlzIGRpc3Rpbmd1aXNoZWQsIG5vdCBv\
        bmx5IGJ5IGhpcyByZWFzb24sIGJ1dCBieSB0aGlzIHNpbmd1bGFyIHBhc3\
        Npb24gZnJvbSBvdGhlciBhbmltYWxzLCB3aGljaCBpcyBhIGx1c3Qgb2Ygd\
        GhlIG1pbmQsIHRoYXQgYnkgYSBwZXJzZXZlcmFuY2Ugb2YgZGVsaWdodCBp\
        biB0aGUgY29udGludWVkIGFuZCBpbmRlZmF0aWdhYmxlIGdlbmVyYXRpb24g\
        b2Yga25vd2xlZGdlLCBleGNlZWRzIHRoZSBzaG9ydCB2ZWhlbWVuY2Ugb2YgY\
        W55IGNhcm5hbCBwbGVhc3VyZS4=";
        assert_eq!(output.as_str(), expected);
    }

    #[test]
    fn test_standard_decode() {
        let input = "";
        let output = base64_decode(input).unwrap();
        assert_eq!(output.as_str(), "");

        let input = "TWFu";
        let output = base64_decode(input).unwrap();
        assert_eq!(output.as_str(), "Man");

        let input = "QQ==";
        let output = base64_decode(input).unwrap();
        assert_eq!(output.as_str(), "A");

        let input = "QkM=";
        let output = base64_decode(input).unwrap();
        assert_eq!(output.as_str(), "BC");

        let input = "ZS4=";
        let output = base64_decode(input).unwrap();
        assert_eq!(output.as_str(), "e.");

        let expected = "\
        Man is distinguished, not only by his reason, \
        but by this singular passion from other animals, \
        which is a lust of the mind, that by a perseverance \
        of delight in the continued and indefatigable generation \
        of knowledge, exceeds the short vehemence of any carnal pleasure.";
        let input = "TWFuIGlzIGRpc3Rpbmd1aXNoZWQsIG5vdCBv\
        bmx5IGJ5IGhpcyByZWFzb24sIGJ1dCBieSB0aGlzIHNpbmd1bGFyIHBhc3\
        Npb24gZnJvbSBvdGhlciBhbmltYWxzLCB3aGljaCBpcyBhIGx1c3Qgb2Ygd\
        GhlIG1pbmQsIHRoYXQgYnkgYSBwZXJzZXZlcmFuY2Ugb2YgZGVsaWdodCBp\
        biB0aGUgY29udGludWVkIGFuZCBpbmRlZmF0aWdhYmxlIGdlbmVyYXRpb24g\
        b2Yga25vd2xlZGdlLCBleGNlZWRzIHRoZSBzaG9ydCB2ZWhlbWVuY2Ugb2YgY\
        W55IGNhcm5hbCBwbGVhc3VyZS4=";
        let output = base64_decode(input).unwrap();
        assert_eq!(output.as_str(), expected);
    }

    #[test]
    fn test_standard_decode_error() {

        let input = "hello";
        let output = base64_decode(input);
        assert_eq!(output.is_err(), true);
        let err = output.unwrap_err();
        assert_eq!(err.cause, Cause::IncorrectPadding);

        let input = "=123";
        let output = base64_decode(input);
        assert_eq!(output.is_err(), true);
        let err = output.unwrap_err();
        assert_eq!(err.cause, Cause::IncorrectPadding);

        let input = "hex123=1";
        let output = base64_decode(input);
        assert_eq!(output.is_err(), true);
        let err = output.unwrap_err();
        assert_eq!(err.cause, Cause::IncorrectPadding);
    }
}
