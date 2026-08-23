use crate::error::Result;

#[derive(Debug, thiserror::Error)]
pub enum BitstreamError {
    #[error("Can't push {0} bits of a u{1}")]
    TooBig(usize, usize),
}

/// A poorly implemented bitstream using a vector of booleans
#[derive(Debug, Default)]
pub struct Bitstream {
    data: Vec<bool>,
}

impl Bitstream {
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn from_bytes(b: &[u8]) -> Self {
        let mut res = Self::new();
        res.push_bytes(b);
        res
    }

    pub fn push(&mut self, data: bool) {
        self.data.push(data);
    }

    pub fn push_u8(&mut self, data: u8, len: usize) -> Result<()> {
        if len > 8 {
            return Err(BitstreamError::TooBig(len, 8).into());
        }
        for i in (0..len).rev() {
            self.data.push((data >> i) & 1 == 1);
        }
        Ok(())
    }

    pub fn push_u16(&mut self, data: u16, len: usize) -> Result<()> {
        if len > 16 {
            return Err(BitstreamError::TooBig(len, 16).into());
        }
        for i in (0..len).rev() {
            self.data.push((data >> i) & 1 == 1);
        }
        Ok(())
    }

    pub fn push_u32(&mut self, data: u32, len: usize) -> Result<()> {
        if len > 32 {
            return Err(BitstreamError::TooBig(len, 32).into());
        }
        for i in (0..len).rev() {
            self.data.push((data >> i) & 1 == 1);
        }
        Ok(())
    }

    pub fn push_bytes(&mut self, data: &[u8]) {
        for b in data {
            let _ = self.push_u8(*b, 8);
        }
    }

    // mostly for testing purposes
    pub fn as_bytes(&self) -> Vec<u8> {
        self.data
            .chunks(8)
            .map(|chunk| {
                let mut res = 0;
                for (i, n) in chunk.iter().enumerate() {
                    res |= (*n as u8) << (7 - i)
                }
                res
            })
            .collect()
    }

    /// length in bytes
    pub fn len(&self) -> usize {
        self.data.len().div_ceil(8)
    }

    pub fn bit_len(&self) -> usize {
        self.data.len()
    }

    /// how many bits free in current byte
    pub fn free_bits(&self) -> usize {
        self.data.len().next_multiple_of(8) - self.data.len()
    }
}

impl From<Bitstream> for Vec<bool> {
    fn from(value: Bitstream) -> Self {
        value.data
    }
}

impl From<Vec<bool>> for Bitstream {
    fn from(value: Vec<bool>) -> Self {
        Self { data: value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitstream_bool() {
        let mut b = Bitstream::new();
        b.push(false);
        b.push(true);
        b.push(false);
        b.push(true);
        b.push(false);
        b.push(true);
        b.push(false);
        b.push(true);

        b.push(true);
        b.push(false);
        b.push(true);

        assert_eq!(b.as_bytes(), vec![0b01010101, 0b10100000])
    }

    #[test]
    fn test_bitstream_u8() {
        let mut b = Bitstream::new();
        let _ = b.push_u8(0xAB, 8);
        let _ = b.push_u8(0xAA, 3);
        assert_eq!(b.as_bytes(), vec![0xAB, 0x40])
    }

    #[test]
    fn test_bitstream_u16() {
        let mut b = Bitstream::new();
        let _ = b.push_u16(0xABCD, 16);
        let _ = b.push_u16(0x0005, 1);
        assert_eq!(b.as_bytes(), vec![0xAB, 0xCD, 0x80])
    }

    #[test]
    fn test_bitstream_u32() {
        let mut b = Bitstream::new();
        let _ = b.push_u32(0xABCDEF12, 32);
        let _ = b.push_u32(0x00000005, 1);
        assert_eq!(b.as_bytes(), vec![0xAB, 0xCD, 0xEF, 0x12, 0x80])
    }
}
