use std::io;

use clap::ValueEnum;
use enum_stringify::EnumStringify;
use image::{Rgb, RgbImage};
use rand::Rng;
use rand::SeedableRng;
use serde::Deserialize;
use serde::Serialize;

/// The mode for Bit changing.
#[derive(Debug, Clone, ValueEnum, Copy, EnumStringify)]
pub enum ChangeMode {
    /// Shifts bits to the left.
    ShiftLeft,
    /// Shifts bits to the right.
    ShiftRight,
    /// Does a NOT operation on the bits.
    Not,
    /// Multiplies the bits.
    Multiply,
    /// Uses the square root of the bits.
    Sqrt,
    /// Does an XOR operation on the bits.
    Xor,
    /// Does an OR operation on the bits.
    Or,
    /// Does an AND operation on the bits.
    And,
    /// Raises the bits to the power of the other provided value
    Exponent,
    /// Adds a random value to the bits, using the other value as a seed.
    RandomAdd,
    /// Multiplies the bits by a random value, using the other value as a seed.
    RandomMul,
}

impl ChangeMode {
    pub fn shift(self, value: u8, other: u32) -> u8 {
        match self {
            Self::ShiftLeft => value.wrapping_shl(other.into()),
            Self::ShiftRight => value.wrapping_shr(other.into()),
            Self::Not => !value,
            Self::Multiply => value.wrapping_mul(other.try_into().unwrap()),
            Self::Sqrt => (value as f32).sqrt() as u8,
            Self::Xor => value ^ other as u8,
            Self::Or => value | other as u8,
            Self::And => value & other as u8,
            Self::Exponent => value.wrapping_pow(other.into()),
            Self::RandomAdd => {
                let mut rng = rand::rngs::SmallRng::seed_from_u64(other as u64);
                value.wrapping_add(rng.gen())
            }
            Self::RandomMul => {
                let mut rng = rand::rngs::SmallRng::seed_from_u64(other as u64);
                value.wrapping_mul(rng.gen())
            }
        }
    }

    pub fn from_string(string: &str) -> Result<Self, String> {
        match string {
            "ShiftLeft" => Ok(ChangeMode::ShiftLeft),
            "ShiftRight" => Ok(ChangeMode::ShiftRight),
            "Not" => Ok(ChangeMode::Not),
            "Multiply" => Ok(ChangeMode::Multiply),
            "Sqrt" => Ok(ChangeMode::Sqrt),
            "Xor" => Ok(ChangeMode::Xor),
            "Or" => Ok(ChangeMode::Or),
            "And" => Ok(ChangeMode::And),
            "Exponent" => Ok(ChangeMode::Exponent),
            "RandomAdd" => Ok(ChangeMode::RandomAdd),
            "RandomMul" => Ok(ChangeMode::RandomMul),
            other => Err(format!("Invalid ChangeMode variant: '{}'", other)),
        }
    }
}

/// The algorithm to use while deepfrying.
#[derive(Debug, Clone)]
pub enum DeepfryAlgorithm {
    /// Changes bits based off a ChangeMode.
    BitChange(ChangeMode, u32, u32, u32),
}

/// Deepfries an image in place.
pub fn deepfry(image: &mut RgbImage, algo: DeepfryAlgorithm) -> io::Result<()> {
    for rgb in image.pixels_mut() {
        let [red, green, blue] = rgb.0;

        let (new_r, new_g, new_b) = match algo {
            DeepfryAlgorithm::BitChange(direction, r, g, b) => {
                let new_red = direction.shift(red, r);
                let new_green = direction.shift(green, g);
                let new_blue = direction.shift(blue, b);
                (new_red, new_green, new_blue)
            }
        };

        *rgb = Rgb([new_r, new_g, new_b])
    }

    Ok(())
}

/// A configuration for an algorithm.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlgorithmConfig {
    pub algorithm: String,
    pub change_mode: Option<String>,
    pub red: Option<u32>,
    pub green: Option<u32>,
    pub blue: Option<u32>,
}

impl AlgorithmConfig {
    pub fn algo(self) -> Result<DeepfryAlgorithm, String> {
        return match self.algorithm.as_str() {
            "BitChange" => {
                if self.change_mode.is_none() {
                    return Err("bit changing mode is not set".to_string());
                }

                let r = self.red.unwrap_or_default();
                let g = self.green.unwrap_or_default();
                let b = self.blue.unwrap_or_default();

                ChangeMode::try_from(self.change_mode.clone().unwrap())
                    .map(|change_mode| DeepfryAlgorithm::BitChange(change_mode, r, g, b))
                    .map_err(|_| format!("invalid bit changing mode {:?}", self.change_mode))
            }
            _ => return Err(format!("invalid algorithm: {}", self.algorithm)),
        };
    }
}

/// A preset for deepfrying images using several algorithms
/// and configs without running multiple commands.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Preset {
    pub algorithms: Vec<AlgorithmConfig>,
}
