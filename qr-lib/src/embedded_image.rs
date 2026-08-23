use image::{DynamicImage, GenericImageView};

pub struct EmbeddedImage {
    data: Vec<Vec<bool>>,
    offset: (i32, i32),
}

impl EmbeddedImage {
    pub fn from_image(
        image: &DynamicImage,
        offset: Option<(i32, i32)>,
        threshold: Option<u8>,
        scale: Option<f64>,
    ) -> Self {
        let offset = offset.unwrap_or((0, 0));
        let threshold = threshold.unwrap_or(128);
        let image = if let Some(scale) = scale {
            let (width, height) = image.dimensions();
            image.resize(
                ((width as f64) * scale) as u32,
                ((height as f64) * scale) as u32,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            image.clone()
        };

        let greyscale = image.to_luma8();
        let (width, height) = greyscale.dimensions();

        let mut data = vec![vec![false; width as usize]; height as usize];
        for (x, y, pixel) in greyscale.enumerate_pixels() {
            let brightness = pixel.0[0];
            data[y as usize][x as usize] = brightness < threshold;
        }

        Self { data, offset }
    }

    pub fn get(&self, row: usize, col: usize) -> bool {
        let (row_offset, col_offset) = self.offset;
        let row_end = self.data.len() as i32 + row_offset;
        let col_end = self.data[0].len() as i32 + col_offset;

        if !(row_offset..row_end).contains(&(row as i32))
            || !(col_offset..col_end).contains(&(col as i32))
        {
            false
        } else {
            self.data[((row as i32) - row_offset) as usize][((col as i32) - col_offset) as usize]
        }
    }
}
