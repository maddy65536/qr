#[derive(Debug)]
pub struct Qr {
    pub data: Vec<Vec<bool>>,
}

impl Qr {
    pub fn make_blank(version: u8) -> Self {
        Self { data: vec![] }
    }

    pub fn make_test_data() -> Self {
        Self {
            data: vec![
                vec![false, true, false, true, false],
                vec![false, true, false, true, false],
                vec![false, false, false, false, false],
                vec![true, false, false, false, true],
                vec![false, true, true, true, false],
            ],
        }
    }
}

pub fn version_to_width(version: u8) -> Option<usize> {
    if version < 1 || version > 40 {
        None
    } else {
        Some(((version * 4) + 17) as usize)
    }
}
