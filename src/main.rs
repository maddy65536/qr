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

struct Options {
    message: String,
    ec: Option<ECLevel>,
    mask: Option<usize>,
    min_version: Option<usize>,
    path: String,
}

fn parse_args(args: Vec<String>) -> Result<Options, String> {
    if args.len() < 2 {
        return Err(HELP_MESSAGE.into());
    }

    let mut args_iter = args.iter().skip(1);
    let Some(message) = args_iter.next() else {
        return Err("no message supplied".into());
    };
    let mut options = Options {
        message: message.clone(),
        ec: None,
        mask: None,
        min_version: None,
        path: String::from("output.bmp"),
    };

    while let Some(option) = args_iter.next() {
        match option.as_str() {
            "--ec" | "-e" => {
                let Some(ec) = args_iter.next() else {
                    return Err("--ec [low|medium|quartile|high]".into());
                };
                options.ec = match ec.as_str() {
                    "low" => Some(ECLevel::Low),
                    "medium" => Some(ECLevel::Medium),
                    "quartile" => Some(ECLevel::Quartile),
                    "high" => Some(ECLevel::High),
                    _ => {
                        return Err("unknown ec level".into());
                    }
                }
            }
            "--mask" | "-m" => {
                let Some(mask_arg) = args_iter.next() else {
                    return Err("--mask [0-7]".into());
                };
                let Ok(mask) = mask_arg.parse() else {
                    return Err("mask must be a number 0-7".into());
                };
                if !(0..=7).contains(&mask) {
                    return Err("mask must be a number 0-7".into());
                }
                options.mask = Some(mask)
            }
            "--min-version" | "-v" => {
                let Some(version_arg) = args_iter.next() else {
                    return Err("--min-version [1-40]".into());
                };
                let Ok(version) = version_arg.parse() else {
                    return Err("min-version must be a number 1-40".into());
                };
                if !(1..=40).contains(&version) {
                    return Err("min-version must be a number 1-40".into());
                }
                options.min_version = Some(version)
            }
            "--output" | "-o" => {
                let Some(path) = args_iter.next() else {
                    return Err("--output (path)".into());
                };
                options.path = path.clone()
            }
            _ => {
                return Err(format!("unknown option: {}", option));
            }
        }
    }

    Ok(options)
}

fn main() {
    let options = match parse_args(std::env::args().collect()) {
        Ok(options) => options,
        Err(msg) => {
            println!("{msg}");
            return;
        }
    };
    let res = qr::Qr::make_qr(
        &options.message,
        options.ec,
        options.mask,
        options.min_version,
    )
    .unwrap();
    let bmp = bitmap::qr_to_bitmap(&res).unwrap();
    std::fs::write(options.path, bmp).unwrap();
}
