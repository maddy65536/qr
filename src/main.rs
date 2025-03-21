use encoding::ECLevel;

mod bitmap;
mod bitstream;
mod encoding;
mod qr;
mod rsec;
mod tables;

fn main() {
    let data = "my qr code generator works :3";
    let res = qr::Qr::make_qr(data, ECLevel::Low).unwrap();
    let bmp = bitmap::qr_to_bitmap(&res).unwrap();
    std::fs::write("test.bmp", bmp).unwrap();
}
