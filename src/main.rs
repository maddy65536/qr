use std::iter;

use encoding::ECLevel;

mod bitmap;
mod bitstream;
mod encoding;
mod qr;
mod rsec;
mod tables;

fn main() {
    let data = "hiii :3";
    let res = qr::Qr::make_qr(data, ECLevel::Low).unwrap();
    let bmp = bitmap::qr_to_bitmap(&res).unwrap();
    std::fs::write("test.bmp", bmp).unwrap();
    // println!("input: {}", data);
    // let mode = encoding::detect_mode(data);
    // let version = encoding::detect_version(mode, data.len());
    // let encoded = encoding::encode(data.into(), mode, version, 19).unwrap();
    // let res = rsec::rs_encode(&encoded, 7);
    // println!("output: {:02X?}", res);

    // for version in 1..=40 {
    //     let test_qr = qr::Qr::make_blank(version, encoding::ECLevel::Low);
    //     let bmp = bitmap::qr_to_bitmap(&test_qr).unwrap();
    //     std::fs::write(format!("images/{}.bmp", version), bmp).unwrap();
    // }

    // let version = 14;
    // let size = qr::version_to_width(version).unwrap();
    // let mut test: Vec<Vec<char>> =
    //     iter::repeat_n(iter::repeat_n('⬜', size).collect(), size).collect();
    // let order = qr::ModuleOrder::new(version);
    // let squares = iter::repeat_n('🟥', 8)
    //     .chain(iter::repeat_n('🟧', 8))
    //     .chain(iter::repeat_n('🟨', 8))
    //     .chain(iter::repeat_n('🟩', 8))
    //     .chain(iter::repeat_n('🟦', 8))
    //     .chain(iter::repeat_n('🟪', 8))
    //     .cycle();
    // order
    //     .zip(squares)
    //     .for_each(|(pos, c)| test[pos.0][pos.1] = c);
    // println!(
    //     "{}",
    //     test.iter()
    //         .map(|row| row.iter().collect::<String>())
    //         .collect::<Vec<String>>()
    //         .join("\n")
    // );
}
