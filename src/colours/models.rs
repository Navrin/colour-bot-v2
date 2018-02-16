use serenity::utils::Colour;

use read_color::rgb;
use colours::names;

use std::fmt::{Display, Error as FmtError, Formatter};

use std::str::FromStr;
use std::error::Error;

use std::f64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColourParseError {
    InvalidFormat,
}

impl ColourParseError {
    pub fn __description(&self) -> &str {
        match self {
            &ColourParseError::InvalidFormat => "Invalid format given.",
        }
    }
}

impl Display for ColourParseError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        self.__description().fmt(fmt)
    }
}

impl Error for ColourParseError {
    fn description(&self) -> &str {
        self.__description()
    }
}

#[derive(Clone, Debug)]
pub struct ParsedColour<'a> {
    pub name: Option<&'a str>,
    pub r: u8,
    pub b: u8,
    pub g: u8,
}

impl<'a> ParsedColour<'a> {
    pub fn into_role_colour(&self) -> Colour {
        Colour::from_rgb(self.r, self.g, self.b)
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn compute_distance(&self, other: &Self) -> f64 {
        f64::abs(
            (other.r as f64 - self.r as f64).powf(2.0) 
            + (other.b as f64 - self.b as f64).powf(2.0)
            + (other.g as f64 - self.g as f64).powf(2.0),
        )
    }

    pub fn to_hex(&self) -> u64 {
        let rgbc = self.b as u64 | (self.g as u64) << 8 | (self.r as u64) << 16;

        rgbc
    }

    pub fn find_nearest(&self, colours: &[Self]) -> Option<Self> {
        let mut expected = None;
        let mut min_distance = f64::INFINITY;

        for colour in colours {
            if colour.to_hex() == self.to_hex() {
                return Some(colour.clone());
            }

            let distance = self.compute_distance(colour);
            if distance < min_distance {
                min_distance = distance;
                expected = Some(colour.clone());
            }
        }

        expected
    }

    pub fn find_name(&self) -> Option<String> {
        let colour = self.find_nearest(&names::COLOUR_NAMES)?;

        colour.name.map(str::to_string)
    }
}

impl<'a> FromStr for ParsedColour<'a> {
    type Err = ColourParseError;

    fn from_str(colour: &str) -> Result<Self, Self::Err> {
        let colour = colour.replace("#", "");
        let mut chars = colour.chars();

        let range = rgb(&mut chars).ok_or(ColourParseError::InvalidFormat)?;

        let r = range[0];
        let g = range[1];
        let b = range[2];

        Ok(ParsedColour {
            r,
            g,
            b,
            name: None,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn can_find_name() {
        let black = ParsedColour::from_str("#000000").unwrap();

        let name = black.find_name().unwrap();

        assert_eq!(name, "Black")
    }
}
