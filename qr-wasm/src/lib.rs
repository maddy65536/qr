use image::DynamicImage;
use qr_lib::{Qr, embedded_image::EmbeddedImage};
use wasm_bindgen::prelude::*;

/// object to hold image so it doesn't need to get uploaded every time
#[wasm_bindgen]
#[derive(Default)]
pub struct QrCodeGenerator {
    image: Option<DynamicImage>,
}

#[wasm_bindgen]
pub struct QrCodeArgs {
    data: String,
    ec: Option<String>,
    mask: Option<usize>,
    min_version: Option<usize>,
}

#[wasm_bindgen]
impl QrCodeArgs {
    #[wasm_bindgen(constructor)]
    pub fn new(
        data: String,
        ec: Option<String>,
        mask: Option<usize>,
        min_version: Option<usize>,
    ) -> Self {
        Self {
            data,
            ec,
            mask,
            min_version,
        }
    }
}

#[wasm_bindgen]
pub struct ImageArgs {
    x_offset: Option<i32>,
    y_offset: Option<i32>,
    threshold: Option<u8>,
    scale: Option<f64>,
}

#[wasm_bindgen]
impl ImageArgs {
    #[wasm_bindgen(constructor)]
    pub fn new(
        x_offset: Option<i32>,
        y_offset: Option<i32>,
        threshold: Option<u8>,
        scale: Option<f64>,
    ) -> Self {
        Self {
            x_offset,
            y_offset,
            threshold,
            scale,
        }
    }
}

#[wasm_bindgen]
pub struct QrCode {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl QrCode {
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

#[wasm_bindgen]
impl QrCodeGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { image: None }
    }

    pub fn set_image(&mut self, data: Vec<u8>) -> Result<(), JsError> {
        let im = image::load_from_memory(&data)
            .map_err(|e| JsError::new(&format!("Image processing error: {e}")))?;
        self.image = Some(im);
        Ok(())
    }

    pub fn generate_qr_code(
        &self,
        qr_args: QrCodeArgs,
        img_args: Option<ImageArgs>,
    ) -> Result<QrCode, JsError> {
        let img = img_args
            .map(|img_args| {
                Ok::<EmbeddedImage, JsError>(EmbeddedImage::from_image(
                    self.image
                        .as_ref()
                        .ok_or(JsError::new("No image provided"))?,
                    Some((
                        img_args.y_offset.unwrap_or(0),
                        img_args.x_offset.unwrap_or(0),
                    )),
                    img_args.threshold,
                    img_args.scale,
                ))
            })
            .transpose()?;

        let qr = Qr::make_qr(
            qr_args.data.as_str(),
            qr_args.ec.map(|s| s.parse()).transpose()?,
            qr_args.mask,
            qr_args.min_version,
            img,
        )?
        .to_image();

        let (width, height) = qr.dimensions();

        Ok(QrCode {
            data: qr.into_raw(),
            width,
            height,
        })
    }
}
