use colours::names;
use read_color::rgb;
use serenity::utils::Colour;
use std::cmp::Ordering;

use std::error::Error;
use std::fmt::{Display, Error as FmtError, Formatter};
use std::str::FromStr;

use hsl::HSL;
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

/// HSL colour represntations.
#[derive(Clone, Debug, PartialEq)]
pub struct HSLColour<'a> {
    pub h: f64,
    pub s: f64,
    pub l: f64,
    pub cmp: HSLCmpType,
    pub name: Option<&'a str>,
    hsl_struct: HSL,
}

impl<'a> HSLColour<'a> {
    pub fn to_parsed(&self) -> ParsedColour<'a> {
        let (r, g, b) = self.hsl_struct.to_rgb();

        ParsedColour {
            r,
            g,
            b,
            name: self.name,
        }
    }
}

impl<'a> Eq for HSLColour<'a> {}

impl<'a> PartialOrd for HSLColour<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<'a> Ord for HSLColour<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.cmp {
            HSLCmpType::Hue => self.h.partial_cmp(&other.h).unwrap_or(Ordering::Less),
        }
    }
}

impl<'a, 'b> From<&'b ParsedColour<'a>> for HSLColour<'a> {
    fn from(parsed: &ParsedColour<'a>) -> Self {
        parsed.to_hsl()
    }
}

// impl<'a> From<HSLColour<'a>> for ParsedColour<'a> {
//     fn from(hsl: HSLColour) -> Self {
//         hsl.to_parsed()
//     }
// }

#[derive(Clone, Debug, PartialEq)]
pub enum HSLCmpType {
    Hue,
}

/// Parsed colour methods.
/// Represents colours for this bot, can be converted to the discord version, also has a naming component.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedColour<'a> {
    pub name: Option<&'a str>,
    pub r: u8,
    pub b: u8,
    pub g: u8,
}

#[derive(Clone, PartialEq, Debug)]
pub enum SortMethod {
    HSL,
}

impl<'a> ParsedColour<'a> {
    pub fn sort_list<T: Into<Self> + Clone>(colours: Vec<T>, method: SortMethod) -> Vec<Self> {
        let colours: Vec<Self> = colours.iter().cloned().map(T::into).collect();

        match method {
            SortMethod::HSL => {
                let mut hsl_list: Vec<HSLColour> = colours.iter().map(HSLColour::from).collect();

                hsl_list.sort_unstable();

                hsl_list.iter().map(|z| z.to_parsed()).collect()
            }

            // SortMethod::Distance => {
            //     colours.sort_by(|a, b| {
            //         a.compute_distance(&b)
            //             .partial_cmp(&b.compute_distance(a))
            //             .unwrap_or(Ordering::Less)
            //     });

            //     colours
            // }
        }
    }

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
        self.b as u64 | (self.g as u64) << 8 | (self.r as u64) << 16
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

    pub fn to_hsl(&self) -> HSLColour<'a> {
        let hsl = HSL::from_rgb(&[self.r, self.g, self.b]);

        HSLColour {
            h: hsl.h,
            s: hsl.s,
            l: hsl.l,
            hsl_struct: hsl,
            name: self.name,
            cmp: HSLCmpType::Hue,
        }
    }
}

impl<'a> From<Colour> for ParsedColour<'a> {
    fn from(colour: Colour) -> Self {
        ParsedColour {
            name: None,
            r: colour.r(),
            g: colour.g(),
            b: colour.b(),
        }
    }
}

impl<'a> Display for ParsedColour<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        // {:02X} means to print in uppercase hexidecimal but pad the output with 0s if its under 2 characters.
        write!(
            f,
            "#{r:02X}{g:02X}{b:02X}",
            r = self.r,
            g = self.g,
            b = self.b
        )
    }
}

impl<'a> FromStr for ParsedColour<'a> {
    type Err = ColourParseError;

    fn from_str(colour: &str) -> Result<Self, Self::Err> {
        let colour = colour.replace("#", "");
        let mut chars = colour.chars();

        let range = if colour.len() == 3 {
            let mut extended_chars: Vec<char> = vec![];

            for ch in chars {
                let mut chs = vec![ch; 2];
                extended_chars.append(&mut chs);
            }

            let extended_chars = extended_chars.iter().collect::<String>();
            let mut chars = extended_chars.chars();

            rgb(&mut chars)
        } else {
            rgb(&mut chars)
        };

        let range = range.ok_or(ColourParseError::InvalidFormat)?;

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
    macro_rules! make_parser_many_case {
        ($cases:expr) => {{
            let case = $cases;

            let iter = case
                .iter()
                .map(|it| ParsedColour::from_str(it));
                
                
            (iter.clone().collect::<Result<Vec<_>, _>>(), iter.collect::<Vec<_>>())
        }};
    }

    use super::*;

    #[test]
    pub fn can_find_name() {
        let black = ParsedColour::from_str("#000000").unwrap();

        let name = black.find_name().unwrap();

        assert_eq!(name, "Black")
    }

    #[test]
    pub fn malformed_colour_codes_fail() {
        let (result, entire) =  make_parser_many_case!(vec![
            "foobaz",
            "#gggZZZ",
            "#fa",
            "BA00gI",
            "not a colour",
            "zzzzzz",
            "!.!.!.",
            "😀😀😀😱😱😱"
        ]);

        assert!(result.is_err(), "A bad colour code passed the test! {:#?}", entire)
    }

    #[test]
    pub fn correct_colour_codes_pass() {
        let (result, entire) = make_parser_many_case!(vec![
            "#cacabb",
            "#cef",
            "ced",
            "123123",
            "#123123",
            "#fafafa",
            "#fac",
            "eee",
            "#eee"
        ]);

        assert!(result.is_ok(), "A good colour failed the test! {:#?}", entire)
    }


    #[test]
    pub fn colours_format_properly() {
        let colour = ParsedColour {
            name: None,
            r: 255,
            g: 0,
            b: 0,
        };

        let output = format!("{}", colour);

        assert_eq!(output, "#FF0000", "The output fmt for the colour {{ r: 255, g: 0, b: 0 }} was incorrect!")
    }

    #[test]
    pub fn colours_translate_properly() {
        let colour_hex = 0xFF00FF;
        let colour_hex_str = "#ff00ff";

        let colour = ParsedColour::from_str(colour_hex_str).unwrap();

        assert_eq!(colour.to_hex(), colour_hex)
    }

    #[test]
    pub fn hex_to_hsl_is_same() {
        let colour = ParsedColour::from_str("#ff00ff").unwrap();

        let hsl = colour.to_hsl();
        
        let new_colour = hsl.to_parsed();

        assert_eq!(colour, new_colour, "Translation between hex->hsl->hex failed!")
    }

    #[test]
    pub fn hsl_sort_is_correct() {
        let codes = vec![
            "#f00",
            "#ffd000",
            "#a1ff00",
            "#00ff2f",
            "#00eaff",
            "#01f",
            "#80f",
            "#ff00e1",
            "#ff0051"
        ];

        let colours = 
            codes
                .iter()
                .map(|x| ParsedColour::from_str(x))
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

        let sorted_colours = 
            ParsedColour::sort_list(colours.clone(), SortMethod::HSL);

        let correctness = 
            sorted_colours
                .iter()
                .zip(colours.iter())
                .map(|(a,b)| a == b)
                .all(|id| id);
        
        assert!(correctness, "There was a disparity between an already sorted set and the sorted output! pre-sorted: {:?}, code-sorted: {:?}", colours, sorted_colours)
                
    }
}
