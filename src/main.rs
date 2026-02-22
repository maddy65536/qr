use clap::Parser;

use qr::{bitmap, encoding::ECLevel, layout};

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
    #[arg(short, long, default_value_t = String::from("output.bmp"))]
    output: String,
}

fn main() {
    let args = Args::parse();
    let res = layout::Qr::make_qr(
        &args.message,
        args.ec,
        args.mask.map(|x| x as usize),
        args.version.map(|x| x as usize),
    )
    .unwrap();
    let bmp = bitmap::qr_to_bitmap(&res).unwrap();
    std::fs::write(args.output, bmp).unwrap();
}
