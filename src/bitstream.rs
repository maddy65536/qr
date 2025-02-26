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

    pub fn push_u8(&mut self, data: u8, len: u8) {
        if len > 8 {
            panic!("can't push {} bits of a u8!", len);
        }
        for i in (0..len).rev() {
            self.data.push((data >> i) & 1 == 1);
        }
    }

    pub fn push_u16(&mut self, data: u16, len: u8) {
        if len > 16 {
            panic!("can't push {} bits of a u16!", len);
        }
        for i in (0..len).rev() {
            self.data.push((data >> i) & 1 == 1);
        }
    }

    pub fn push_u32(&mut self, data: u32, len: u8) {
        if len > 32 {
            panic!("can't push {} bits of a u32!", len);
        }
        for i in (0..len).rev() {
            self.data.push((data >> i) & 1 == 1);
        }
    }

    pub fn push_bytes(&mut self, data: &[u8]) {
        for b in data {
            self.push_u8(*b, 8);
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

    /// how many bits free in current byte
    pub fn free_bits(&self) -> usize {
        self.data.len() % 8
    }
}

impl From<Bitstream> for Vec<bool> {
    fn from(value: Bitstream) -> Self {
        value.data
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
        b.push_u8(0xAB, 8);
        b.push_u8(0xAA, 3);
        assert_eq!(b.as_bytes(), vec![0xAB, 0x40])
    }

    #[test]
    fn test_bitstream_u16() {
        let mut b = Bitstream::new();
        b.push_u16(0xABCD, 16);
        b.push_u16(0x0005, 1);
        assert_eq!(b.as_bytes(), vec![0xAB, 0xCD, 0x80])
    }

    #[test]
    fn test_bitstream_u32() {
        let mut b = Bitstream::new();
        b.push_u32(0xABCDEF12, 32);
        b.push_u32(0x00000005, 1);
        assert_eq!(b.as_bytes(), vec![0xAB, 0xCD, 0xEF, 0x12, 0x80])
    }
}
