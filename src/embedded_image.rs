use image::DynamicImage;

pub struct EmbeddedImage {
    data: Vec<Vec<bool>>,
    row_offset: usize,
    col_offset: usize,
}

impl EmbeddedImage {
    pub fn from_image(image: DynamicImage, row_offset: usize, col_offset: usize) -> Self {
        let greyscale = image.to_luma8();
        let (width, height) = greyscale.dimensions();
        let mut data = vec![vec![false; width as usize]; height as usize];
        let threshold = 128;
        for (x, y, pixel) in greyscale.enumerate_pixels() {
            let brightness = pixel.0[0];
            data[y as usize][x as usize] = brightness < threshold;
        }

        Self {
            data,
            row_offset,
            col_offset,
        }
    }

    pub fn get(&self, row: usize, col: usize) -> bool {
        if !((row.wrapping_sub(self.row_offset))..(self.data.len().wrapping_sub(self.row_offset)))
            .contains(&row)
            || !((col.wrapping_sub(self.col_offset))
                ..(self.data[0].len().wrapping_sub(self.col_offset)))
                .contains(&col)
        {
            false
        } else {
            self.data[row - self.row_offset][col - self.col_offset]
        }
    }
}
