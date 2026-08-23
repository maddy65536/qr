mod bitstream;
pub mod embedded_image;
pub mod encoding;
pub mod error;
mod layout;
mod rsec;
mod tables;

use std::iter;

use image::{ImageBuffer, Rgba, RgbaImage};

use crate::embedded_image::EmbeddedImage;
use crate::encoding::ECLevel;
use crate::error::{Error, Result};
use crate::layout::{ModuleOrder, apply_best_mask, make_fixed_patterns, score_matrix};

// fixed mask pattern for embedded images
pub const EMBEDDED_IMAGE_MASK: usize = 7;
// output scale
const SCALE: usize = 4;

#[derive(Debug, Clone)]
pub struct Qr {
    pub data: Vec<Vec<bool>>,
    version: usize,
    ec: ECLevel,
}

impl Qr {
    pub fn make_blank(version: usize, ec: ECLevel) -> Result<Self> {
        if !(1..=40).contains(&version) {
            return Err(Error::InvalidVersion(version));
        }
        Ok(Self {
            data: make_fixed_patterns(version)?,
            version,
            ec,
        })
    }

    pub fn make_qr(
        data: &str,
        ec: Option<ECLevel>,
        mask: Option<usize>,
        min_version: Option<usize>,
        image: Option<EmbeddedImage>,
    ) -> Result<Self> {
        let ec = ec.unwrap_or(ECLevel::Low);
        println!("ec level: {:?}", ec);
        let min_version = min_version.unwrap_or(0);
        // if there's an image to embed use the override mask
        let mask = if image.is_some() {
            Some(EMBEDDED_IMAGE_MASK)
        } else {
            mask
        };
        // encode data
        let mode = encoding::detect_mode(data);
        println!("mode: {:?}", mode);
        let version = encoding::detect_version(mode, encoding::data_len(mode, data), ec)
            .expect("too much data")
            .max(min_version);
        println!("version: {:?}", version);
        let encoded = encoding::encode(data, mode, version, ec, image)?;
        let stream: Vec<bool> = bitstream::Bitstream::from_bytes(&encoded).into();

        // draw qr code
        let mut qr = Self::make_blank(version, ec)?;
        let order = ModuleOrder::new(version)?;
        stream
            .iter()
            .zip(order)
            .for_each(|(bit, pos)| qr.data[pos.0][pos.1] = *bit);

        qr = apply_best_mask(&qr, mask)?;
        Ok(qr)
    }

    pub fn score(&self) -> usize {
        score_matrix(&self.data)
    }

    pub fn to_image(&self) -> RgbaImage {
        // finalize layout and scaling
        let mut res: Vec<Vec<bool>> = iter::repeat_n(
            iter::repeat_n(false, (self.data[0].len() + 8) * SCALE).collect(),
            (self.data.len() + 8) * SCALE,
        )
        .collect();
        for (i, row) in res
            .iter_mut()
            .enumerate()
            .take((self.data[0].len() + 4) * SCALE)
            .skip(4 * SCALE)
        {
            for (j, module) in row
                .iter_mut()
                .enumerate()
                .take((self.data.len() + 4) * SCALE)
                .skip(4 * SCALE)
            {
                *module = self.data[(i / SCALE) - 4][(j / SCALE) - 4];
            }
        }

        let mut im: RgbaImage = ImageBuffer::new(res[0].len() as u32, res.len() as u32);
        for (x, y, pixel) in im.enumerate_pixels_mut() {
            let module = res[y as usize][x as usize];
            *pixel = if module {
                Rgba([0, 0, 0, 255])
            } else {
                Rgba([255, 255, 255, 255])
            };
        }
        im
    }
}
