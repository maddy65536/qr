use crate::qr::Qr;
use std::iter;

const BMP_HEADER_LEN: usize = 54;
const SCALE: usize = 4;

pub fn qr_to_bitmap(qr: &Qr) -> Option<Vec<u8>> {
    let mut res: Vec<Vec<bool>> = iter::repeat_n(
        iter::repeat_n(false, (qr.data[0].len() + 8) * SCALE).collect(),
        (qr.data.len() + 8) * SCALE,
    )
    .collect();
    for (i, row) in res
        .iter_mut()
        .enumerate()
        .take((qr.data[0].len() + 4) * SCALE)
        .skip(4 * SCALE)
    {
        for (j, module) in row
            .iter_mut()
            .enumerate()
            .take((qr.data.len() + 4) * SCALE)
            .skip(4 * SCALE)
        {
            *module = qr.data[(i / SCALE) - 4][(j / SCALE) - 4];
        }
    }
    make_bitmap(&res)
}

pub fn make_bitmap(data: &[Vec<bool>]) -> Option<Vec<u8>> {
    if data.is_empty() || data[0].is_empty() {
        return None;
    }

    let height = data.len();
    let width = data[0].len();
    let pixel_length: usize = data.iter().map(|v| (v.len() * 3).next_multiple_of(4)).sum();
    let result_length = BMP_HEADER_LEN + pixel_length;
    let mut res = vec![];

    // header
    res.extend_from_slice(b"BM"); // magic number
    res.extend_from_slice(&(result_length as u32).to_le_bytes()); // size
    res.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // reserved
    res.extend_from_slice(&[0x36, 0x00, 0x00, 0x00]); // pixel array offset

    // other header lol
    res.extend_from_slice(&[0x28, 0x00, 0x00, 0x00]); // DIB header size
    res.extend_from_slice(&(width as u32).to_le_bytes()); // width
    res.extend_from_slice(&(height as u32).to_le_bytes()); // height
    res.extend_from_slice(&[0x01, 0x00]); // planes
    res.extend_from_slice(&[0x18, 0x00]); // bits per pixel
    res.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // compression
    res.extend_from_slice(&(pixel_length as u32).to_le_bytes()); // image size
    res.extend_from_slice(&[0x23, 0x2E, 0x00, 0x00]); // x pixels per meter
    res.extend_from_slice(&[0x23, 0x2E, 0x00, 0x00]); // y pixels per meter
    res.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // colors in color table
    res.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // important color count

    // wheeeeee
    let pixel_data: Vec<u8> = data
        .iter()
        .rev()
        .map(|r| {
            r.iter()
                .flat_map(|b| if *b { [0, 0, 0] } else { [255, 255, 255] })
        })
        .flat_map(|r| r.chain(iter::repeat_n(0, width % 4)))
        .collect();
    res.extend_from_slice(&pixel_data);

    Some(res)
}
