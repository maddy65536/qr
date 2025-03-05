mod bitmap;
mod bitstream;
mod encoding;
mod qr;
mod rsec;
mod tables;

fn main() {
    let data = "hiii :3";
    println!("input: {}", data);
    let mode = encoding::detect_mode(data);
    let version = encoding::detect_version(mode, data.len());
    let encoded = encoding::encode(data.into(), mode, version, 19).unwrap();
    let res = rsec::rs_encode(&encoded, 7);
    println!("output: {:02X?}", res);

    for version in 1..=40 {
        let test_qr = qr::Qr::make_blank(version);
        let bmp = bitmap::qr_to_bitmap(&test_qr).unwrap();
        std::fs::write(format!("images/{}.bmp", version), bmp).unwrap();
    }
}
