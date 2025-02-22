/// A poorly implemented bitstream using a vector of booleans
pub struct Bitstream {
    data: Vec<bool>,
}

impl Bitstream {
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn push(&mut self, data: bool) {
        self.data.push(data);
    }

    pub fn push_u8(&mut self, data: u8, len: u8) {
        if len > 8 {
            panic!("can't push {} bits of a u8!", len);
        }
        for i in ((8 - len)..8).rev() {
            self.data.push((data >> i) & 1 == 1);
        }
    }

    pub fn push_u16(&mut self, data: u16, len: u8) {
        if len > 16 {
            panic!("can't push {} bits of a u16!", len);
        }
        for i in ((16 - len)..16).rev() {
            self.data.push((data >> i) & 1 == 1);
        }
    }

    pub fn push_u32(&mut self, data: u32, len: u8) {
        if len > 32 {
            panic!("can't push {} bits of a u32!", len);
        }
        for i in ((32 - len)..32).rev() {
            self.data.push((data >> i) & 1 == 1);
        }
    }

    // mostly for testing purposes
    pub fn into_bytes(self) -> Vec<u8> {
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
}

impl From<Vec<bool>> for Bitstream {
    fn from(value: Vec<bool>) -> Self {
        Self { data: value }
    }
}

impl Into<Vec<bool>> for Bitstream {
    fn into(self) -> Vec<bool> {
        self.data
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

        assert_eq!(b.into_bytes(), vec![0b01010101, 0b10100000])
    }

    #[test]
    fn test_bitstream_u8() {
        let mut b = Bitstream::new();
        b.push_u8(0xAB, 8);
        b.push_u8(0xAA, 3);
        assert_eq!(b.into_bytes(), vec![0xAB, 0xA0])
    }

    #[test]
    fn test_bitstream_u16() {
        let mut b = Bitstream::new();
        b.push_u16(0xABCD, 16);
        b.push_u16(0xA000, 1);
        assert_eq!(b.into_bytes(), vec![0xAB, 0xCD, 0x80])
    }

    #[test]
    fn test_bitstream_u32() {
        let mut b = Bitstream::new();
        b.push_u32(0xABCDEF12, 32);
        b.push_u32(0xA0000000, 1);
        assert_eq!(b.into_bytes(), vec![0xAB, 0xCD, 0xEF, 0x12, 0x80])
    }
}
