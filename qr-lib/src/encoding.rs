use clap::ValueEnum;
use encoding_rs::SHIFT_JIS;

use std::{collections::VecDeque, str::FromStr};

use crate::{
    EMBEDDED_IMAGE_MASK,
    bitstream::Bitstream,
    embedded_image::EmbeddedImage,
    error::{Error, Result},
    layout::ModuleOrder,
    rsec,
    tables::{ALPHANUMERIC_ORDER, BLOCK_GROUPS, DATA_CAPACITY, LENGTH_BITS},
};

#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    #[error("Invalid char for alphanumeric mode: {0}")]
    InvalidAlphanumChar(char),
    #[error("Failed to encode to shift-jis: {0}")]
    ShiftJisEncode(String),
    #[error("Shift-jis char out of range: {0:#x}")]
    ShiftJisOutOfRange(u16),
    #[error("Non numeric characters in numeric encoding")]
    NonNumericInNumeric,
    #[error("Too much data!")]
    TooBig,
    #[error("Group depletion")]
    GroupDepletion,
    #[error("Invalid EC level: {0}")]
    EcLevel(String),
}

#[allow(unused)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Numeric = 0b0001,
    Alphanumeric = 0b0010,
    Byte = 0b0100,
    Kanji = 0b1000,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, ValueEnum)]
pub enum ECLevel {
    Low = 0b01,
    Medium = 0b00,
    Quartile = 0b11,
    High = 0b10,
}

impl FromStr for ECLevel {
    type Err = Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s {
            "low" => Ok(ECLevel::Low),
            "medium" => Ok(ECLevel::Medium),
            "quartile" => Ok(ECLevel::Quartile),
            "high" => Ok(ECLevel::High),
            l => Err(EncodeError::EcLevel(l.to_owned()).into()),
        }
    }
}

pub fn detect_mode(data: &str) -> Mode {
    if is_numeric(data) {
        Mode::Numeric
    } else if is_alphanumeric(data) {
        Mode::Alphanumeric
    } else if is_kanji(data) {
        Mode::Kanji
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

// this feels inefficient
fn is_kanji(data: &str) -> bool {
    data.chars().all(|c| {
        let s = c.to_string();
        let (encoded, _, error) = SHIFT_JIS.encode(&s);
        if error || encoded.len() < 2 {
            return false;
        }
        let value = (encoded[0] as u16) << 8 | (encoded[1] as u16);
        (0x8140..=0x9FFC).contains(&value) || (0xE040..=0xEBBF).contains(&value)
    })
}

pub fn get_length_bits(mode: Mode, version: usize) -> Result<usize> {
    let index = match version {
        1..=9 => 0,
        10..=26 => 1,
        27..=40 => 2,
        _ => return Err(Error::InvalidVersion(version)),
    };
    Ok(LENGTH_BITS[(mode as u32).ilog2() as usize][index])
}

pub fn data_len(mode: Mode, data: &str) -> usize {
    let len = data.len();
    match mode {
        Mode::Numeric => {
            (len / 3) * 10 + ((len % 3 == 1) as usize) * 4 + ((len % 3 == 2) as usize) * 7
        }
        Mode::Alphanumeric => ((len / 2) * 11) + ((len & 1) * 6),
        Mode::Byte => len * 8,
        Mode::Kanji => data.chars().count() * 13,
    }
}

// find smallest version that fits data
pub fn detect_version(mode: Mode, len: usize, ec: ECLevel) -> Result<usize> {
    for (v, row) in DATA_CAPACITY.iter().enumerate() {
        let capacity = row[ec as usize] * 8;
        // 8 extra bits for mode selector + terminator
        let size = (8 + get_length_bits(mode, v + 1)? + len).next_multiple_of(8);
        if size <= capacity {
            return Ok(v + 1);
        }
    }
    Err(EncodeError::TooBig.into())
}

fn char_to_alphanum(data: char) -> Result<u16> {
    Ok(ALPHANUMERIC_ORDER
        .iter()
        .position(|c| *c == data)
        .ok_or(EncodeError::InvalidAlphanumChar(data))? as u16)
}

pub fn encode(
    data: &str,
    mode: Mode,
    version: usize,
    ec: ECLevel,
    image: Option<EmbeddedImage>,
) -> Result<Vec<u8>> {
    let num_codewords = DATA_CAPACITY[version - 1][ec as usize];

    let mut res = Bitstream::new();

    // mode indicator
    res.push_u8(mode as u8, 4)?;

    // length indicator
    let len = if let Mode::Kanji = mode {
        data.chars().count()
    } else {
        data.len()
    };
    res.push_u16(len as u16, get_length_bits(mode, version)?)?;

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
                res.push_u16(
                    chunk
                        .parse()
                        .map_err(|_| EncodeError::NonNumericInNumeric)?,
                    len,
                )?;
            }
        }
        Mode::Alphanumeric => {
            let mut chars = data.chars().peekable();
            while chars.peek().is_some() {
                let chunk: Vec<char> = chars.by_ref().take(2).collect();
                if chunk.len() == 1 {
                    res.push_u16(char_to_alphanum(chunk[0])?, 6)?;
                } else {
                    let code = (45 * char_to_alphanum(chunk[0])?) + char_to_alphanum(chunk[1])?;
                    res.push_u16(code, 11)?;
                }
            }
        }
        Mode::Byte => {
            for b in data.as_bytes() {
                res.push_u8(*b, 8)?;
            }
        }
        Mode::Kanji => {
            let (encoded, _, error) = SHIFT_JIS.encode(data);
            if error {
                return Err(EncodeError::ShiftJisEncode(data.to_owned()).into());
            }

            // at this point, we know the data should entirely consist of 2 byte characters
            for chunk in encoded.chunks(2) {
                let c = (chunk[0] as u16) << 8 | (chunk[1] as u16);

                let subtraction_value = if (0x8140..=0x9FFC).contains(&c.into()) {
                    0x8140
                } else if (0xE040..=0xEBBF).contains(&c.into()) {
                    0xC140
                } else {
                    return Err(EncodeError::ShiftJisOutOfRange(c).into());
                };

                let subtracted = c - subtraction_value;
                let upper = (subtracted >> 8) & 0xFF;
                let lower = subtracted & 0xFF;
                let encoded = (upper * 0xC0) + lower;
                res.push_u16(encoded, 13)?;
            }
        }
    }

    res.push_u8(0, 4)?; // insert terminator
    res.push_u8(0, res.free_bits())?; // fill remaining bits in last byte

    // insert padding or image
    let padding: Vec<u8> = if let Some(image) = image {
        let positions: Vec<(usize, usize)> = get_final_order(version, ec)?;
        let start = res.bit_len();
        let len = (num_codewords * 8) - start;
        let stream: Bitstream = positions
            .into_iter()
            .skip(start)
            .take(len)
            .map(|(row, col)| {
                image.get(row, col) != crate::layout::MASKS[EMBEDDED_IMAGE_MASK]((row, col))
            })
            .collect::<Vec<bool>>()
            .into();

        stream.as_bytes()
    } else {
        [0xEC, 0x11]
            .into_iter()
            .cycle()
            .take(num_codewords - res.len())
            .collect()
    };
    res.push_bytes(&padding);

    let res_bytes = res.as_bytes();
    interleave_and_ec(&res_bytes, version, ec)
}

fn interleave_and_ec(bytes: &[u8], version: usize, ec: ECLevel) -> Result<Vec<u8>> {
    let mut groups: Vec<VecDeque<u8>> = vec![];
    let mut ec_groups: Vec<VecDeque<u8>> = vec![];
    let mut res: Vec<u8> = vec![];

    let mut bytes_iter = bytes.iter().cloned();
    // group 1
    let ((num_ec_blocks, num_blocks, block_size), _) = BLOCK_GROUPS[version - 1][ec as usize];
    for _ in 0..num_blocks {
        let group: Vec<u8> = (&mut bytes_iter).take(block_size).collect();
        let ec_group = rsec::rs_encode(&group, num_ec_blocks)?[group.len()..].to_vec();
        groups.push(group.into());
        ec_groups.push(ec_group.into());
    }

    // group 2
    if let (_, Some((num_ec_blocks, num_blocks, block_size))) =
        BLOCK_GROUPS[version - 1][ec as usize]
    {
        for _ in 0..num_blocks {
            let group: Vec<u8> = (&mut bytes_iter).take(block_size).collect();
            let ec_group = rsec::rs_encode(&group, num_ec_blocks)?[group.len()..].to_vec();
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
                res.push(group.pop_front().ok_or(EncodeError::GroupDepletion)?);
            }
        }
    }

    finished = false;
    while !finished {
        finished = true;
        for group in ec_groups.iter_mut() {
            if !group.is_empty() {
                finished = false;
                res.push(group.pop_front().ok_or(EncodeError::GroupDepletion)?);
            }
        }
    }

    Ok(res)
}

// really janky
fn get_final_order(version: usize, ec: ECLevel) -> Result<Vec<(usize, usize)>> {
    let mut groups: Vec<VecDeque<usize>> = vec![];
    let mut res: Vec<usize> = vec![];
    let indicies: Vec<usize> = (0..DATA_CAPACITY[version - 1][ec as usize]).collect();

    let mut indicies_iter = indicies.iter().cloned();
    // group 1
    let ((_, num_blocks, block_size), _) = BLOCK_GROUPS[version - 1][ec as usize];
    for _ in 0..num_blocks {
        let group: Vec<usize> = (&mut indicies_iter).take(block_size).collect();
        groups.push(group.into());
    }

    // group 2
    if let (_, Some((_, num_blocks, block_size))) = BLOCK_GROUPS[version - 1][ec as usize] {
        for _ in 0..num_blocks {
            let group: Vec<usize> = (&mut indicies_iter).take(block_size).collect();
            groups.push(group.into());
        }
    }

    // build result
    let mut finished = false;
    while !finished {
        finished = true;
        for group in groups.iter_mut() {
            if !group.is_empty() {
                finished = false;
                res.push(group.pop_front().ok_or(EncodeError::GroupDepletion)?);
            }
        }
    }

    let order = ModuleOrder::new(version)?.collect::<Vec<_>>();
    let mut final_order = vec![(0, 0); res.len() * 8];
    for (i, data_index) in res.iter().enumerate().take(final_order.len()) {
        let base = i * 8;
        let data_base = *data_index * 8;
        final_order[data_base] = order[base];
        final_order[data_base + 1] = order[base + 1];
        final_order[data_base + 2] = order[base + 2];
        final_order[data_base + 3] = order[base + 3];
        final_order[data_base + 4] = order[base + 4];
        final_order[data_base + 5] = order[base + 5];
        final_order[data_base + 6] = order[base + 6];
        final_order[data_base + 7] = order[base + 7];
    }

    Ok(final_order)
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
        assert_eq!(detect_mode("一二三四五六七八九十"), Mode::Kanji);
    }

    #[test]
    fn test_get_length_bits() {
        assert_eq!(get_length_bits(Mode::Numeric, 1).unwrap(), 10);
        assert_eq!(get_length_bits(Mode::Alphanumeric, 15).unwrap(), 11);
        assert_eq!(get_length_bits(Mode::Byte, 29).unwrap(), 16);
        assert_eq!(get_length_bits(Mode::Kanji, 14).unwrap(), 10);
    }

    #[test]
    fn test_data_len() {
        assert_eq!(data_len(Mode::Byte, "aaaa"), 32);
        assert_eq!(data_len(Mode::Numeric, "123456"), 20);
        assert_eq!(data_len(Mode::Numeric, "1234567"), 24);
        assert_eq!(data_len(Mode::Numeric, "12345678"), 27);
        assert_eq!(data_len(Mode::Alphanumeric, "ABC1"), 22);
        assert_eq!(data_len(Mode::Alphanumeric, "ABC12"), 28);
        assert_eq!(data_len(Mode::Kanji, "のワの"), 39);
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
            )
            .unwrap(),
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
