mod bitstream;
mod encoding;
mod rsec;
mod tables;

use bitstream::Bitstream;
use rsec::rs_encode;

fn main() {
    let data = vec![
        0x40, 0x77, 0x46, 0x57, 0x37, 0x42, 0x03, 0xA3, 0x30, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11,
        0xEC, 0x11, 0xEC, 0x11,
    ];
    let res = rs_encode(&data, 7);
    println!("input:  {:02X?}", data);
    println!("result: {:02X?}", res);

    let b = Bitstream::new();
    let c: Vec<bool> = b.into();
}
