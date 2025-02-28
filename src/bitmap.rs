use crate::qr::Qr;
use std::iter;

const BMP_HEADER_LEN: usize = 54;

pub fn qr_to_bitmap(qr: &Qr) -> Option<Vec<u8>> {
    make_bitmap(&qr.data)
}

pub fn make_bitmap(data: &[Vec<bool>]) -> Option<Vec<u8>> {
    if data.is_empty() || data[0].is_empty() {
        return None;
    }

    let height = data.len();
    let width = data[0].len();
    let pixel_length = data
        .iter()
        .map(|v| (v.len() * 3).next_multiple_of(4))
        .sum::<usize>();
    let result_length: usize = BMP_HEADER_LEN + pixel_length;
    let mut res = vec![];

    // header
    res.extend_from_slice(b"BM"); // magic number
    res.extend_from_slice(&(result_length as u32).to_le_bytes()); // size
    res.extend_from_slice(b"\x00\x00\x00\x00"); // reserved
    res.extend_from_slice(b"\x36\x00\x00\x00"); // pixel array offset

    // other header lol
    res.extend_from_slice(b"\x28\x00\x00\x00"); // DIB header size
    res.extend_from_slice(&(width as u32).to_le_bytes()); // width
    res.extend_from_slice(&(height as u32).to_le_bytes()); // height
    res.extend_from_slice(b"\x01\x00"); // planes
    res.extend_from_slice(b"\x18\x00"); // bits per pixel
    res.extend_from_slice(b"\x00\x00\x00\x00"); // compression
    res.extend_from_slice(&(pixel_length as u32).to_le_bytes()); // image size
    res.extend_from_slice(&[0xE8, 0x03, 0x00, 0x00]); // x pixels per meter
    res.extend_from_slice(&[0xE8, 0x03, 0x00, 0x00]); // y pixels per meter
    res.extend_from_slice(b"\x00\x00\x00\x00"); // colors in color table
    res.extend_from_slice(b"\x00\x00\x00\x00"); // important color count

    // wheeeeee
    let pixel_data: Vec<u8> = data
        .iter()
        .rev()
        .map(|r| {
            r.iter()
                .flat_map(|b| if *b { [0, 0, 0] } else { [255, 255, 255] })
                .collect::<Vec<u8>>()
        })
        .flat_map(|r| [r, iter::repeat_n(0, width % 4).collect()])
        .flatten()
        .collect();
    res.extend_from_slice(&pixel_data);

    Some(res)
}
