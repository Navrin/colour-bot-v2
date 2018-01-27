use serenity::utils::Colour;

use read_color::rgb;

use std::fmt::{Display, Error as FmtError, Formatter};

use std::str::FromStr;
use std::error::Error;

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

pub struct ParsedColour {
    pub r: u8,
    pub b: u8,
    pub g: u8,
}

impl ParsedColour {
    pub fn into_role_colour(&self) -> Colour {
        Colour::from_rgb(self.r, self.g, self.b)
    }
}

impl FromStr for ParsedColour {
    type Err = ColourParseError;

    fn from_str(colour: &str) -> Result<Self, Self::Err> {
        let colour = colour.replace("#", "");
        let mut chars = colour.chars();

        let range = rgb(&mut chars).ok_or(ColourParseError::InvalidFormat)?;

        let r = range[0];
        let g = range[1];
        let b = range[2];

        Ok(ParsedColour { r, g, b })
    }
}
