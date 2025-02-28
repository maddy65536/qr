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

    let test_qr = qr::Qr::make_test_data();
    let bmp = bitmap::qr_to_bitmap(&test_qr).unwrap();
    println!("{:02X?}", bmp);
    std::fs::write("test.bmp", bmp).unwrap();
}
