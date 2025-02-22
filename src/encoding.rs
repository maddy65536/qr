// ignoring structured apend for now
enum Mode {
    Numeric = 0b0001,
    Alphanumeric = 0b0010,
    Byte = 0b0100,
    Kanji = 0b1000,
    Eci = 0b0111,
}

pub fn encode(data: String) -> Vec<u8> {
    let mode = if data.is_ascii() {
        Mode::Byte
    } else {
        todo!("implement other modes")
    };

    vec![]
}
