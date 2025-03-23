use std::collections::VecDeque;

use crate::{
    bitstream::Bitstream,
    rsec,
    tables::{ALPHANUMERIC_ORDER, BLOCK_GROUPS, DATA_CAPACITY, LENGTH_BITS},
};

// ignoring structured apend for now
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Numeric = 0b0001,
    Alphanumeric = 0b0010,
    Byte = 0b0100,
    Kanji = 0b1000,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ECLevel {
    Low = 0b01,
    Medium = 0b00,
    Quartile = 0b11,
    High = 0b10,
}

// should add kanji mode and potentially support for mixing modes
pub fn detect_mode(data: &str) -> Mode {
    if is_numeric(data) {
        Mode::Numeric
    } else if is_alphanumeric(data) {
        Mode::Alphanumeric
    } else {
        Mode::Byte
    }
}

fn is_numeric(data: &str) -> bool {
    data.chars().all(|c| c.is_ascii() && c.is_numeric())
}

fn is_alphanumeric(data: &str) -> bool {
    data.chars().all(|c| ALPHANUMERIC_ORDER.contains(&c))
}

pub fn get_length_bits(mode: Mode, version: usize) -> Option<usize> {
    let index = match version {
        1..=9 => 0,
        10..=26 => 1,
        27..=40 => 2,
        _ => return None,
    };
    Some(LENGTH_BITS[(mode as u32).ilog2() as usize][index])
}

pub fn data_len(mode: Mode, len: usize) -> usize {
    match mode {
        Mode::Numeric => {
            (len / 3) * 10 + ((len % 3 == 1) as usize) * 4 + ((len % 3 == 2) as usize) * 7
        }
        Mode::Alphanumeric => ((len / 2) * 11) + ((len & 1) * 6),
        Mode::Byte => len * 8,
        Mode::Kanji => unimplemented!(),
    }
}

// find smallest version that fits data
pub fn detect_version(mode: Mode, len: usize, ec: ECLevel) -> Option<usize> {
    for (v, row) in DATA_CAPACITY.iter().enumerate() {
        let capacity = row[ec as usize] * 8;
        // 8 extra bits for mode selector + terminator
        let size = (8 + get_length_bits(mode, v + 1)? + len).next_multiple_of(8);
        if size <= capacity {
            return Some(v + 1);
        }
    }
    None
}

fn char_to_alphanum(data: char) -> u16 {
    ALPHANUMERIC_ORDER.iter().position(|c| *c == data).unwrap() as u16
}

pub fn encode(data: &str, mode: Mode, version: usize, ec: ECLevel) -> Option<Vec<u8>> {
    let num_codewords = DATA_CAPACITY[version - 1][ec as usize];
    println!("num codewords: {}", num_codewords);

    let mut res = Bitstream::new();

    // mode indicator
    res.push_u8(mode as u8, 4);

    // length indicator
    res.push_u16(data.len() as u16, get_length_bits(mode, version)?);

    match mode {
        Mode::Numeric => {
            let mut chars = data.chars().peekable();
            while chars.peek().is_some() {
                let chunk: String = chars.by_ref().take(3).collect();
                let len = if chunk.len() == 1 {
                    4
                } else if chunk.len() == 2 {
                    7
                } else {
                    10
                };
                res.push_u16(chunk.parse().unwrap(), len);
            }
        }
        Mode::Alphanumeric => {
            let mut chars = data.chars().peekable();
            while chars.peek().is_some() {
                let chunk: Vec<char> = chars.by_ref().take(2).collect();
                if chunk.len() == 1 {
                    res.push_u16(char_to_alphanum(chunk[0]), 6);
                } else {
                    let code = (45 * char_to_alphanum(chunk[0])) + char_to_alphanum(chunk[1]);
                    res.push_u16(code, 11);
                }
            }
        }
        Mode::Byte => {
            for b in data.as_bytes() {
                res.push_u8(*b, 8);
            }
        }
        Mode::Kanji => unimplemented!(),
    }

    res.push_u8(0, 4); // insert terminator
    res.push_u8(0, res.free_bits()); // fill remaining bits in last byte

    // insert padding
    let padding: Vec<u8> = [0xEC, 0x11]
        .into_iter()
        .cycle()
        .take(num_codewords - res.len())
        .collect();
    res.push_bytes(&padding);

    let res_bytes = res.as_bytes();
    Some(interleave_and_ec(&res_bytes, version, ec))
}

fn interleave_and_ec(bytes: &[u8], version: usize, ec: ECLevel) -> Vec<u8> {
    let mut groups: Vec<VecDeque<u8>> = vec![];
    let mut ec_groups: Vec<VecDeque<u8>> = vec![];
    let mut res: Vec<u8> = vec![];

    let mut bytes_iter = bytes.iter().cloned();
    // group 1
    let ((num_ec_blocks, num_blocks, block_size), _) = BLOCK_GROUPS[version - 1][ec as usize];
    for _ in 0..num_blocks {
        let group: Vec<u8> = (&mut bytes_iter).take(block_size).collect();
        let ec_group = rsec::rs_encode(&group, num_ec_blocks)[group.len()..].to_vec();
        groups.push(group.into());
        ec_groups.push(ec_group.into());
    }

    // group 2
    if let (_, Some((num_ec_blocks, num_blocks, block_size))) =
        BLOCK_GROUPS[version - 1][ec as usize]
    {
        for _ in 0..num_blocks {
            let group: Vec<u8> = (&mut bytes_iter).take(block_size).collect();
            let ec_group = rsec::rs_encode(&group, num_ec_blocks)[group.len()..].to_vec();
            groups.push(group.into());
            ec_groups.push(ec_group.into());
        }
    }

    // build result
    let mut finished = false;
    while !finished {
        finished = true;
        for group in groups.iter_mut() {
            if !group.is_empty() {
                finished = false;
                res.push(group.pop_front().unwrap());
            }
        }
    }

    finished = false;
    while !finished {
        finished = true;
        for group in ec_groups.iter_mut() {
            if !group.is_empty() {
                finished = false;
                res.push(group.pop_front().unwrap());
            }
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_mode() {
        assert_eq!(detect_mode("123456"), Mode::Numeric);
        assert_eq!(detect_mode("123456ABC"), Mode::Alphanumeric);
        assert_eq!(detect_mode("123456ABCabc'!%&"), Mode::Byte);
        assert_eq!(detect_mode("123456ABCDEFabcdef'!%&¥"), Mode::Byte);
        assert_eq!(detect_mode("一二三四五六七八九十"), Mode::Byte);
    }

    #[test]
    fn test_get_length_bits() {
        assert_eq!(get_length_bits(Mode::Numeric, 1), Some(10));
        assert_eq!(get_length_bits(Mode::Alphanumeric, 15), Some(11));
        assert_eq!(get_length_bits(Mode::Byte, 29), Some(16));
        assert_eq!(get_length_bits(Mode::Kanji, 14), Some(10));
    }

    #[test]
    fn test_data_len() {
        assert_eq!(data_len(Mode::Byte, 4), 32);
        assert_eq!(data_len(Mode::Numeric, 6), 20);
        assert_eq!(data_len(Mode::Numeric, 7), 24);
        assert_eq!(data_len(Mode::Numeric, 8), 27);
        assert_eq!(data_len(Mode::Alphanumeric, 4), 22);
        assert_eq!(data_len(Mode::Alphanumeric, 5), 28);
    }

    #[test]
    fn test_interleave() {
        assert_eq!(
            interleave_and_ec(
                &[
                    0x41, 0x14, 0x86, 0x56, 0xC6, 0xC6, 0xF2, 0xC2, 0x07, 0x76, 0xF7, 0x26, 0xC6,
                    0x42, 0x12, 0x03, 0x13, 0x23, 0x30, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC,
                    0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11,
                    0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC,
                    0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC,
                ],
                5,
                ECLevel::Quartile
            ),
            vec![
                0x41, 0x03, 0x11, 0x11, 0x14, 0x13, 0xEC, 0xEC, 0x86, 0x23, 0x11, 0x11, 0x56, 0x30,
                0xEC, 0xEC, 0xC6, 0xEC, 0x11, 0x11, 0xC6, 0x11, 0xEC, 0xEC, 0xF2, 0xEC, 0x11, 0x11,
                0xC2, 0x11, 0xEC, 0xEC, 0x07, 0xEC, 0x11, 0x11, 0x76, 0x11, 0xEC, 0xEC, 0xF7, 0xEC,
                0x11, 0x11, 0x26, 0x11, 0xEC, 0xEC, 0xC6, 0xEC, 0x11, 0x11, 0x42, 0x11, 0xEC, 0xEC,
                0x12, 0xEC, 0x11, 0x11, 0xEC, 0xEC, 0x4A, 0x55, 0x87, 0x87, 0x83, 0xF3, 0x93, 0x93,
                0x59, 0x98, 0x07, 0x07, 0x2F, 0xEE, 0x29, 0x29, 0x66, 0xA5, 0x80, 0x80, 0x25, 0x27,
                0x96, 0x96, 0xBB, 0xC8, 0x78, 0x78, 0xCF, 0xED, 0xB8, 0xB8, 0x37, 0x9F, 0x25, 0x25,
                0xAF, 0xBE, 0xB5, 0xB5, 0xC2, 0xB1, 0xCD, 0xCD, 0x7F, 0x23, 0xDE, 0xDE, 0x6B, 0x09,
                0xE7, 0xE7, 0xC1, 0x7A, 0x08, 0x08, 0x9D, 0x9C, 0x2C, 0x2C, 0xD1, 0xD9, 0x51, 0x51,
                0x41, 0x38, 0xAD, 0xAD, 0x89, 0xD8, 0x50, 0x50,
            ]
        )
    }
}
