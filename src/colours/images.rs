use std::fs::File;
use std::io::Error as IoError;
use std::path::PathBuf;

use colours::{models::SortMethod, ParsedColour};
use serenity::utils::Colour;

use cairo::IoError as CairoIoError;

use resvg;
use resvg::Error as ReSvgError;
use svg;

use svg::node::element::{Group, Rectangle, Text as TextEl};
use svg::node::Text;
use svg::Document;

const DARK_THEME_BACKGROUND: &str = "#36393e";

const TOP_PADDING: usize = 50;

const FONT_SIZE: f64 = 35.0;
const LIST_ROW_HEIGHT: usize = 65;
const LIST_COLUMN_WIDTH: usize = 500;

/// Opaque name type to prevent wrong items being entered into the list.
#[derive(Clone, Debug)]
pub struct Name(pub String);

#[derive(Clone)]
pub enum ColourListType {
    BasicList,
}

#[derive(Debug)]
pub enum ColourBuilderError {
    CairoIo(CairoIoError),
    Io(IoError),
    ReSvg(ReSvgError),
}

#[derive(Debug)]
pub struct ColourSection {
    pub name: String,
    pub colour: Colour,
    pub y: f64,
    pub x: f64,
}

impl From<IoError> for ColourBuilderError {
    fn from(e: IoError) -> Self {
        ColourBuilderError::Io(e)
    }
}

impl From<CairoIoError> for ColourBuilderError {
    fn from(e: CairoIoError) -> Self {
        ColourBuilderError::CairoIo(e)
    }
}

impl From<ReSvgError> for ColourBuilderError {
    fn from(e: ReSvgError) -> Self {
        ColourBuilderError::ReSvg(e)
    }
}

/// In the rust rewrite, the colour bot now uses an SVG based render system using cario.
/// *TODO:* Figure out how to make cario work on windows, currently cant compile atm, find out a solution and put it in the docs
pub struct ColourListBuilder {
    list_type: ColourListType,
    dual_colour: bool,
    // show_hex_codes: bool,
}

impl ColourListBuilder {
    pub fn new() -> ColourListBuilder {
        ColourListBuilder {
            list_type: ColourListType::BasicList,
            dual_colour: true,
            // show_hex_codes: true,
        }
    }

    // pub fn set_type(&mut self, cl: ColourListType) -> &mut ColourListBuilder {
    //     self.list_type = cl;
    //     self
    // }

    // pub fn set_dual_colour(&mut self, st: bool) -> &mut ColourListBuilder {
    //     self.dual_colour = st;
    //     self
    // }

    // pub fn set_show_hex_code(&mut self, st: bool) -> &mut ColourListBuilder {
    //     self.show_hex_codes = st;
    //     self
    // }

    pub fn get_height_for_type(&self, amount: usize) -> usize {
        match self.list_type {
            ColourListType::BasicList => amount * LIST_ROW_HEIGHT as usize,
        }
    }

    pub fn get_width_for_type(&self, columns: usize) -> usize {
        match self.list_type {
            ColourListType::BasicList => LIST_COLUMN_WIDTH * columns,
        }
    }

    pub fn get_section_from_colour(
        &self,
        name: String,
        colour: Colour,
        height: usize,
        colours: &Vec<(Name, Colour)>,
    ) -> ColourSection {
        match self.list_type {
            ColourListType::BasicList => {
                let top_margin = if height != colours.len() {
                    TOP_PADDING as f64
                } else {
                    0.0
                };

                let full_height = height as f64 * LIST_ROW_HEIGHT as f64;

                ColourSection {
                    colour,
                    name,
                    y: full_height + top_margin,
                    x: 10.0,
                }
            }
        }
    }

    pub fn generate_image_for_list(
        &self,
        document: Document,
        colours: Vec<ColourSection>,
        (height, width, _): (usize, usize, usize),
    ) -> Document {
        let background = Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", width)
            .set("height", height)
            .set("fill", DARK_THEME_BACKGROUND);

        let document = if self.dual_colour {
            let second = Rectangle::new()
                .set("x", LIST_COLUMN_WIDTH)
                .set("y", 0)
                .set("width", LIST_COLUMN_WIDTH)
                .set("height", height)
                .set("fill", "white");

            document.add(background).add(second)
        } else {
            document.add(background)
        };

        let list = colours
            .iter()
            .map(|section| {
                let (r, g, b) = section.colour.tuple();
                let path = Group::new()
                    .set("x", section.x)
                    .set("y", section.y)
                    .set("width", LIST_COLUMN_WIDTH)
                    .set("height", LIST_ROW_HEIGHT);

                let text = TextEl::new()
                    .set("x", section.x)
                    .set("y", section.y)
                    .set("width", LIST_COLUMN_WIDTH)
                    .set("font-family", "Roboto")
                    .set("font-size", FONT_SIZE)
                    .set("fill", format!("rgb({}, {}, {})", r, g, b))
                    .add(Text::new(section.name.clone()));

                if self.dual_colour {
                    let second_text = text
                        .clone()
                        .set("x", section.x as usize + LIST_COLUMN_WIDTH);

                    path.add(second_text).add(text)
                } else {
                    path.add(text)
                }
            })
            .collect::<Vec<Group>>();

        list.iter().fold(document, |doc, now| doc.add(now.clone()))
    }

    pub fn transform_colours_to_sections(
        &self,
        colours: Vec<(Name, Colour)>,
    ) -> Vec<ColourSection> {
        colours
            .iter()
            .zip(0..colours.len())
            .map(|(&(ref name, colour), height)| {
                self.get_section_from_colour(name.0.clone(), colour, height, &colours)
            })
            .collect::<Vec<ColourSection>>()
    }

    fn sort_colours(&self, colours: Vec<(Name, Colour)>) -> Vec<(Name, Colour)> {
        // get config for the sort type

        // cheat a little, replace the name in the colour struct with the given name

        let colours_remade: Vec<ParsedColour> = colours
            .iter()
            .map(|(name, colour)| ParsedColour {
                name: Some(&name.0),
                ..ParsedColour::from(*colour)
            })
            .collect();

        ParsedColour::sort_list(colours_remade, SortMethod::HSL)
            .iter()
            .map(|colour| {
                (
                    Name(colour.name.unwrap().to_string()),
                    colour.into_role_colour(),
                )
            })
            .collect()
    }

    pub fn create_image<S: Into<String>>(
        &self,
        colours: Vec<(Name, Colour)>,
        id: S,
    ) -> Result<PathBuf, ColourBuilderError> {
        let columns = if self.dual_colour { 2 } else { 1 };
        let height = self.get_height_for_type(colours.len());
        let width = self.get_width_for_type(columns);
        let sorted_colors = self.sort_colours(colours);

        let colours = self.transform_colours_to_sections(sorted_colors);

        let doc = svg::Document::new().set("viewBox", (0, 0, width, height));

        let doc = match self.list_type {
            ColourListType::BasicList => {
                self.generate_image_for_list(doc, colours, (height, width, columns))
            }
        };

        let opt = resvg::Options::default();
        let svg = resvg::parse_doc_from_data(&format!("{}", doc), &opt)?;
        let surface = resvg::render_cairo::render_to_image(&svg, &opt)?;

        let mut path = PathBuf::new();
        path.push(id.into());
        path.set_extension("png");

        let mut file = File::create(&path)?;
        surface.write_to_png(&mut file)?;

        Ok(path)
    }
}
