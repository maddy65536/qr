use crate::{bitstream::Bitstream, tables::LENGTH_BITS};

// ignoring structured apend for now
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Numeric = 0b0001,
    Alphanumeric = 0b0010,
    Byte = 0b0100,
    Kanji = 0b1000,
    Eci = 0b0111,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ECLevel {
    Low,
    Medium,
    Quartile,
    High,
}

// should add kanji mode and potentially support for mixing modes
pub fn detect_mode(data: &str) -> Mode {
    if data.chars().all(|c| c.is_ascii() && c.is_numeric()) {
        Mode::Numeric
    } else if data
        .chars()
        .all(|c| c.is_ascii() && (c.is_uppercase() || c.is_numeric()))
    {
        Mode::Alphanumeric
    } else if data.is_ascii() {
        Mode::Byte
    } else {
        Mode::Eci
    }
}

pub fn get_length_bits(mode: Mode, version: u8) -> Option<u8> {
    let index = match version {
        1..=9 => 0,
        10..=26 => 1,
        27..=40 => 2,
        _ => return None,
    };
    Some(LENGTH_BITS[(mode as u32).ilog2() as usize][index])
}

// todo: finish this lol
pub fn detect_version(mode: Mode, len: usize) -> u8 {
    1
}

pub fn encode(data: String, mode: Mode, version: u8, num_codewords: usize) -> Option<Vec<u8>> {
    if mode != Mode::Byte {
        todo!("i'll get to it")
    }

    let mut res = Bitstream::new();

    // mode indicator
    res.push_u8(mode as u8, 4);

    // length indicator
    res.push_u16(data.len() as u16, get_length_bits(mode, version)?);

    // just handling byte mode for now
    for b in data.as_bytes() {
        res.push_u8(*b, 8);
    }

    res.push_u8(0, 4); // insert terminator
    res.push_u8(0, res.free_bits() as u8); // fill remaining bits in last byte

    // insert padding
    let padding: Vec<u8> = [0xEC, 0x11]
        .into_iter()
        .cycle()
        .take(num_codewords - res.len())
        .collect();
    res.push_bytes(&padding);

    Some(res.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_mode() {
        assert_eq!(detect_mode("123456".into()), Mode::Numeric);
        assert_eq!(detect_mode("123456ABC".into()), Mode::Alphanumeric);
        assert_eq!(detect_mode("123456ABCabc'!%&".into()), Mode::Byte);
        assert_eq!(detect_mode("123456ABCDEFabcdef'!%&¥".into()), Mode::Eci);
        assert_eq!(detect_mode("一二三四五六七八九十".into()), Mode::Eci);
    }

    #[test]
    fn test_get_length_bits() {
        assert_eq!(get_length_bits(Mode::Numeric, 1), Some(10));
        assert_eq!(get_length_bits(Mode::Alphanumeric, 15), Some(11));
        assert_eq!(get_length_bits(Mode::Byte, 29), Some(16));
        assert_eq!(get_length_bits(Mode::Eci, 2), Some(8));
        assert_eq!(get_length_bits(Mode::Kanji, 14), Some(10));
    }
}
