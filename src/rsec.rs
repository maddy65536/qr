// based on code from https://en.wikiversity.org/wiki/Reed%E2%80%93Solomon_codes_for_coders
use crate::tables::{GF_EXP, GF_LOG};

const QR_FORMAT_GENERATOR: usize = 0x537;
const QR_FORMAT_MASK: usize = 0b101010000010010;

pub fn gf_add(x: u8, y: u8) -> u8 {
    x ^ y
}

pub fn gf_sub(x: u8, y: u8) -> u8 {
    x ^ y
}

pub fn gf_mul(x: u8, y: u8) -> u8 {
    if x == 0 || y == 0 {
        0
    } else {
        GF_EXP[(GF_LOG[x as usize] + GF_LOG[y as usize]) % 255]
    }
}

pub fn gf_div(x: u8, y: u8) -> u8 {
    if x == 0 {
        0
    } else if y == 0 {
        panic!("attempt to divide by zero")
    } else {
        GF_EXP[(GF_LOG[x as usize] + 255 - GF_LOG[y as usize]) % 255]
    }
}

pub fn poly_mul(x: &[u8], y: &[u8]) -> Vec<u8> {
    let mut res = vec![0u8; x.len() + y.len() - 1];

    for j in 0..y.len() {
        for i in 0..x.len() {
            res[i + j] ^= gf_mul(x[i], y[j]);
        }
    }

    res
}

// this kinda sucks, maybe make a lookup table?
pub fn rs_generator_poly(num_ec_blocks: usize) -> Vec<u8> {
    let mut res = vec![1];
    for exp in GF_EXP.iter().take(num_ec_blocks).cloned() {
        let curr = vec![1, exp];
        res = poly_mul(&res, &curr);
    }
    res
}

pub fn rs_encode(data: &[u8], num_ec_blocks: usize) -> Vec<u8> {
    if data.len() + num_ec_blocks > 255 {
        panic!(
            "message too long! was {} but max is 255!",
            data.len() + num_ec_blocks
        )
    }

    let gen_poly = rs_generator_poly(num_ec_blocks);

    let mut res = vec![0; data.len() + gen_poly.len() - 1];
    res[..data.len()].copy_from_slice(data);

    for i in 0..data.len() {
        let coef = res[i];
        if coef != 0 {
            for j in 1..gen_poly.len() {
                res[i + j] ^= gf_mul(gen_poly[j], coef)
            }
        }
    }

    res[..data.len()].copy_from_slice(data);
    res
}

pub fn qr_format_check(fmt: usize) -> usize {
    let mut res = fmt;
    for i in (0..=4).rev() {
        if (res & (1 << (i + 10))) != 0 {
            res ^= QR_FORMAT_GENERATOR << i;
        }
    }
    res
}

pub fn qr_format_encode(fmt: usize) -> usize {
    if fmt > 0b11111 {
        panic!("tried to encode invalid format!")
    }
    ((fmt as usize) << 10) | qr_format_check((fmt as usize) << 10)
}

pub fn qr_format_encode_masked(fmt: usize) -> usize {
    qr_format_encode(fmt) ^ QR_FORMAT_MASK
}

#[cfg(test)]
mod tests {
    use super::*;

    // i don't really need a test for this but it's here for completeness
    #[test]
    fn test_gf_add_sub() {
        assert_eq!(gf_add(0b0101, 0b0110), 0b011);
        assert_eq!(gf_sub(0b0101, 0b0110), 0b011);
    }

    #[test]
    fn test_gf_mul() {
        assert_eq!(gf_mul(0, 0b00101010), 0);
        assert_eq!(gf_mul(0b10001001, 0), 0);
        assert_eq!(gf_mul(0b10001001, 0b00101010), 0b11000011);
    }

    #[test]
    fn test_gf_div() {
        assert_eq!(gf_div(0, 0b00101010), 0);
        assert_eq!(gf_div(0b10001001, 0b00101010), 0b11011100);
    }

    #[test]
    #[should_panic]
    fn test_gf_div_zero() {
        let _ = gf_div(0b10001001, 0);
    }

    #[test]
    fn test_poly_mul() {
        let x = vec![0x01, 0x02, 0x03];
        let y = vec![0x04, 0x05, 0x06];
        assert_eq!(poly_mul(&x, &y), vec![0x04, 0x0D, 0x00, 0x03, 0x0A]);
    }

    #[test]
    fn test_poly_mul_weird() {
        let x = vec![1];
        let y = vec![1, 1];
        assert_eq!(poly_mul(&x, &y), vec![1, 1]);
    }

    #[test]
    fn test_gen_poly() {
        assert_eq!(
            rs_generator_poly(16),
            vec![
                1, 59, 13, 104, 189, 68, 209, 30, 8, 163, 65, 41, 229, 98, 50, 36, 59
            ]
        )
    }

    #[test]
    fn test_encode() {
        let data = vec![
            0x40, 0xD2, 0x75, 0x47, 0x76, 0x17, 0x32, 0x06, 0x27, 0x26, 0x96, 0xC6, 0xC6, 0x96,
            0x70, 0xEC,
        ];
        let res = rs_encode(&data, 10);
        assert_eq!(
            res,
            vec![
                0x40, 0xD2, 0x75, 0x47, 0x76, 0x17, 0x32, 0x06, 0x27, 0x26, 0x96, 0xC6, 0xC6, 0x96,
                0x70, 0xEC, 0xBC, 0x2A, 0x90, 0x13, 0x6B, 0xAF, 0xEF, 0xFD, 0x4B, 0xE0,
            ]
        )
    }

    #[test]
    fn test_encode_2() {
        let data = vec![
            0x40, 0x77, 0x46, 0x57, 0x37, 0x42, 0x03, 0xA3, 0x30, 0xEC, 0x11, 0xEC, 0x11, 0xEC,
            0x11, 0xEC, 0x11, 0xEC, 0x11,
        ];
        let res = rs_encode(&data, 7);
        assert_eq!(
            res,
            vec![
                0x40, 0x77, 0x46, 0x57, 0x37, 0x42, 0x03, 0xA3, 0x30, 0xEC, 0x11, 0xEC, 0x11, 0xEC,
                0x11, 0xEC, 0x11, 0xEC, 0x11, 0xD7, 0x39, 0xC0, 0x0C, 0x03, 0x43, 0x5C,
            ]
        )
    }

    #[test]
    fn test_format_encode() {
        assert_eq!(qr_format_encode(0b00011), 0b000111101011001)
    }
}
