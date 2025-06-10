use std::{error::Error, fs, path::PathBuf};

use clap::Parser;
use clap_num::number_range;
use deepfry::{deepfry, AlgorithmConfig, ChangeMode, DeepfryAlgorithm::BitChange, Preset};

fn parse_shift_value(s: &str) -> Result<u32, String> {
    number_range(s, 0, u32::MAX)
}

/// Deepfry - A tool for deepfrying images.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The input file.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// The output file.
    #[arg(value_name = "output")]
    output: PathBuf,

    /// The red shift.
    #[arg(short = 'r', value_name = "red", value_parser=parse_shift_value, default_value = "1")]
    red: u32,

    /// The green shift.
    #[arg(short = 'g', value_name = "green", value_parser=parse_shift_value, default_value = "1")]
    green: u32,

    /// The blue shift.
    #[arg(short = 'b', value_name = "blue", value_parser=parse_shift_value, default_value = "1")]
    blue: u32,

    /// The bit changing mode.
    #[arg(short = 'm', value_name = "mode")]
    mode: Option<ChangeMode>,

    /// The preset.
    #[arg(short = 'p', value_name = "preset")]
    preset: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut image = image::open(args.input).unwrap().to_rgb8();

    assert!(
        args.preset.is_some() || args.mode.is_some(),
        "no bit change mode or preset specified"
    );
    

    if let Some(preset) = args.preset {
        let content = &fs::read_to_string(preset)?;

        let preset: Preset = toml::from_str(content)?;
        preset
            .algorithms
            .into_iter()
            .map(AlgorithmConfig::algo)
            .map(|result| {
                result.unwrap_or_else(|err| panic!("error while validating preset {err}"))
            })
            .try_for_each(|algo| deepfry(&mut image, algo))?
    } else {
        deepfry(
            &mut image,
            BitChange(args.mode.unwrap(), args.red, args.green, args.blue),
        )?;
    }

    image.save(args.output).expect("failed to save image");
    Ok(())
}
