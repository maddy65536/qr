#[derive(Debug)]
pub struct Qr {
    data: Vec<Vec<bool>>,
}

impl Qr {
    fn make_blank(version: u8) -> Self {
        Self { data: vec![] }
    }
}
