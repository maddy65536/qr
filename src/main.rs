mod bitmap;
mod bitstream;
mod encoding;
mod qr;
mod rsec;
mod tables;

use encoding::ECLevel;

const HELP_MESSAGE: &str = "Usage: qr \"message\" [options]

options:
\t-e / --ec [low|medium|quartile|high]
\t-m / --mask [0-7]
\t-v / --min-version [1-40]
\t-o / --output (path)";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("{HELP_MESSAGE}");
        return;
    }

    let mut args_iter = args.iter().skip(1);
    let message = args_iter.next().unwrap();
    let mut ec = None;
    let mut mask = None;
    let mut min_version = None;
    let mut path = "output.bmp";

    while let Some(option) = args_iter.next() {
        match option.as_str() {
            "--ec" | "-e" => {
                ec = match args_iter
                    .next()
                    .expect("--ec [low|medium|quartile|high]")
                    .as_str()
                {
                    "low" => Some(ECLevel::Low),
                    "medium" => Some(ECLevel::Medium),
                    "quartile" => Some(ECLevel::Quartile),
                    "high" => Some(ECLevel::High),
                    _ => {
                        println!("unknown ec level");
                        return;
                    }
                }
            }
            "--mask" | "-m" => {
                let m = args_iter
                    .next()
                    .expect("--mask [0-7]")
                    .parse()
                    .expect("mask must be a number 0-7");
                if !(0..=7).contains(&m) {
                    println!("mask must be a number 0-7");
                    return;
                }
                mask = Some(m)
            }
            "--min-version" | "-v" => {
                let m = args_iter
                    .next()
                    .expect("--min-version [1-40]")
                    .parse()
                    .expect("min-version must be a number 1-40");
                if !(1..=40).contains(&m) {
                    println!("min-version must be a number 1-40");
                    return;
                }
                min_version = Some(m)
            }
            "--output" | "-o" => path = args_iter.next().expect("--output (path)"),
            _ => {
                println!("unknown option: {}", option);
                return;
            }
        }
    }

    let res = qr::Qr::make_qr(message, ec, mask, min_version).unwrap();
    let bmp = bitmap::qr_to_bitmap(&res).unwrap();
    std::fs::write(path, bmp).unwrap();
}
