use std::path::PathBuf;

use clap::Parser;

use qr_lib::{Qr, embedded_image::EmbeddedImage, encoding::ECLevel};

#[derive(Debug, Parser)]
struct Args {
    /// Message to encode
    message: String,

    /// Set Error Correction level
    #[arg(short, long, value_enum)]
    ec: Option<ECLevel>,

    /// Force mask pattern [0-7]
    #[arg(short, long, value_parser = clap::value_parser!(u64).range(0..=7))]
    mask: Option<u64>,

    /// Force minimum version [1-40]
    #[arg(short, long, value_parser = clap::value_parser!(u64).range(1..=40))]
    version: Option<u64>,

    /// Output path
    #[arg(short, long, default_value_t = String::from("output.png"))]
    output: String,

    /// Image: file path
    #[arg(short, long)]
    image_path: Option<PathBuf>,

    /// Image: x offset
    #[arg(long, requires = "image_path")]
    image_x_offset: Option<i32>,

    /// Image: y offset
    #[arg(long, requires = "image_path")]
    image_y_offset: Option<i32>,

    /// Image: threshold
    #[arg(long, requires = "image_path")]
    image_threshold: Option<u8>,

    /// Image: scale
    #[arg(long, requires = "image_path")]
    image_scale: Option<f64>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let image = if let Some(path) = args.image_path {
        Some(EmbeddedImage::from_image(
            &image::ImageReader::open(path)?.decode()?,
            Some((
                args.image_y_offset.unwrap_or(0),
                args.image_x_offset.unwrap_or(0),
            )),
            args.image_threshold,
            args.image_scale,
        ))
    } else {
        None
    };

    let res = Qr::make_qr(
        &args.message,
        args.ec,
        args.mask.map(|x| x as usize),
        args.version.map(|x| x as usize),
        image,
    )?;
    res.to_image().save(args.output)?;

    Ok(())
}
