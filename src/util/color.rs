use material_color_utilities_rs::{scheme::scheme::Scheme, util::color::format_argb_as_rgb};
use owo_colors::{OwoColorize, Style};

use prettytable::{format, Cell, Row, Table};

use crate::Schemes;

use crate::util::image::fetch_image;

use color_eyre::Help;
use image::ImageError;

use super::arguments::{ColorFormat, Format, Source};
use super::image::source_color_from_image;
use color_eyre::{eyre::Result, Report};
use colorsys::{ColorAlpha, Hsl, Rgb};
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;

pub const COLORS: [&str; 46] = [
    "source_color",
    "primary",
    "primary_fixed",
    "primary_fixed_dim",
    "on_primary",
    "on_primary_fixed",
    "on_primary_fixed_variant",
    "primary_container",
    "on_primary_container",
    "secondary",
    "secondary_fixed",
    "secondary_fixed_dim",
    "on_secondary",
    "on_secondary_fixed",
    "on_secondary_fixed_variant",
    "secondary_container",
    "on_secondary_container",
    "tertiary",
    "tertiary_fixed",
    "tertiary_fixed_dim",
    "on_tertiary",
    "on_tertiary_fixed",
    "on_tertiary_fixed_variant",
    "tertiary_container",
    "on_tertiary_container",
    "error",
    "on_error",
    "error_container",
    "on_error_container",
    "surface",
    "on_surface",
    "on_surface_variant",
    "outline",
    "outline_variant",
    "shadow",
    "scrim",
    "inverse_surface",
    "inverse_on_surface",
    "inverse_primary",
    "surface_dim",
    "surface_bright",
    "surface_container_lowest",
    "surface_container_low",
    "surface_container",
    "surface_container_high",
    "surface_container_highest",
];

// TODO Fix this monstrosity
pub trait SchemeExt {
    fn get_value<'a>(&'a self, field: &str, source_color: &'a [u8; 4]) -> &[u8; 4];
}
impl SchemeExt for Scheme {
    fn get_value<'a>(&'a self, field: &str, source_color: &'a [u8; 4]) -> &[u8; 4] {
        match field {
            "primary" => &self.primary,
            "primary_fixed" => &self.primary_fixed,
            "primary_fixed_dim" => &self.primary_fixed_dim,
            "on_primary" => &self.on_primary,
            "on_primary_fixed" => &self.on_primary_fixed,
            "on_primary_fixed_variant" => &self.on_primary_fixed_variant,
            "primary_container" => &self.primary_container,
            "on_primary_container" => &self.on_primary_container,
            "secondary" => &self.secondary,
            "secondary_fixed" => &self.secondary_fixed,
            "secondary_fixed_dim" => &self.secondary_fixed_dim,
            "on_secondary" => &self.on_secondary,
            "on_secondary_fixed" => &self.on_secondary_fixed,
            "on_secondary_fixed_variant" => &self.on_secondary_fixed_variant,
            "secondary_container" => &self.secondary_container,
            "on_secondary_container" => &self.on_secondary_container,
            "tertiary" => &self.tertiary,
            "tertiary_fixed" => &self.tertiary_fixed,
            "tertiary_fixed_dim" => &self.tertiary_fixed_dim,
            "on_tertiary" => &self.on_tertiary,
            "on_tertiary_fixed" => &self.on_tertiary_fixed,
            "on_tertiary_fixed_variant" => &self.on_tertiary_fixed_variant,
            "tertiary_container" => &self.tertiary_container,
            "on_tertiary_container" => &self.on_tertiary_container,
            "error" => &self.error,
            "on_error" => &self.on_error,
            "error_container" => &self.error_container,
            "on_error_container" => &self.on_error_container,
            "surface" => &self.surface,
            "on_surface" => &self.on_surface,
            "on_surface_variant" => &self.on_surface_variant,
            "outline" => &self.outline,
            "outline_variant" => &self.outline_variant,
            "shadow" => &self.shadow,
            "scrim" => &self.scrim,
            "inverse_surface" => &self.inverse_surface,
            "inverse_on_surface" => &self.inverse_on_surface,
            "inverse_primary" => &self.inverse_primary,
            "surface_dim" => &self.surface_dim,
            "surface_bright" => &self.surface_bright,
            "surface_container_lowest" => &self.surface_container_lowest,
            "surface_container_low" => &self.surface_container_low,
            "surface_container" => &self.surface_container,
            "surface_container_high" => &self.surface_container_high,
            "surface_container_highest" => &self.surface_container_highest,
            "source_color" => source_color,
            _ => panic!("{}", field),
        }
    }
}

#[derive(Debug)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    pub fn new(colors: [u8; 4]) -> Color {
        Color {
            red: colors[1],
            green: colors[2],
            blue: colors[3],
            alpha: colors[0],
        }
    }
}

pub fn show_color(schemes: &Schemes, source_color: &[u8; 4]) {
    let mut table: Table = generate_table_format();

    for field in COLORS {
        let color_light: Color =
            Color::new(*Scheme::get_value(&schemes.light, field, source_color));
        let color_dark: Color = Color::new(*Scheme::get_value(&schemes.dark, field, source_color));

        generate_table_rows(&mut table, field, color_light, color_dark);
    }

    table.printstd();
}

fn hex(color: Color, prefix: bool) -> String {
    format!(
        "{}{:02x}{:02x}{:02x}",
        if prefix { "#" } else { "" },
        color.red,
        color.green,
        color.blue
    )
}

pub fn dump_json(schemes: &Schemes, source_color: &[u8; 4], format: &Format) {
    type F = Format;
    let fmt = match format {
        F::Rgb => |c: Color| format!("rgb({:?}, {:?}, {:?})", c.red, c.green, c.blue),
        F::Rgba => |c: Color| {
            format!(
                "rgba({:?}, {:?}, {:?}, {:?})",
                c.red, c.green, c.blue, c.alpha
            )
        },
        F::Hsl => {
            |c: Color| Hsl::new(c.red as f64, c.green as f64, c.blue as f64, None).to_css_string()
        }
        F::Hsla => |c: Color| {
            Hsl::new(
                c.red as f64,
                c.green as f64,
                c.blue as f64,
                Some(c.alpha as f64),
            )
            .to_css_string()
        },
        F::Hex => |c: Color| hex(c, true),
        F::Strip => |c: Color| hex(c, false),
    };

    let mut colors_normal_light: HashMap<&str, String> = HashMap::new();
    let mut colors_normal_dark: HashMap<&str, String> = HashMap::new();
    let mut colors_normal_amoled: HashMap<&str, String> = HashMap::new();

    for field in COLORS {
        let color_light: Color =
            Color::new(*Scheme::get_value(&schemes.light, field, source_color));
        let color_dark: Color = Color::new(*Scheme::get_value(&schemes.dark, field, source_color));

        colors_normal_light.insert(field, fmt(color_light));
        colors_normal_dark.insert(field, fmt(color_dark));
    }

    println!(
        "{}",
        json!({
            "colors": {
                "light": colors_normal_light,
                "dark": colors_normal_dark,
            },
        })
    );
}

fn generate_table_format() -> Table {
    let mut table = Table::new();
    let format = format::FormatBuilder::new()
        .column_separator('│')
        .borders('│')
        .separators(
            &[format::LinePosition::Title],
            format::LineSeparator::new('─', '┼', '├', '┤'),
        )
        .separators(
            &[format::LinePosition::Top],
            format::LineSeparator::new('─', '┬', '╭', '╮'),
        )
        .separators(
            &[format::LinePosition::Bottom],
            format::LineSeparator::new('─', '┴', '╰', '╯'),
        )
        .padding(1, 1)
        .build();

    table.set_format(format);

    table.set_titles(Row::new(vec![
        Cell::new("NAME").style_spec("c"),
        Cell::new("LIGHT").style_spec("c"),
        Cell::new("LIGHT").style_spec("c"),
        Cell::new("DARK").style_spec("c"),
        Cell::new("DARK").style_spec("c"),
    ]));
    table
}

fn generate_table_rows(table: &mut Table, field: &str, color_light: Color, color_dark: Color) {
    let formatstr = "  ";

    table.add_row(Row::new(vec![
        // Color names
        Cell::new(field).style_spec(""),
        // Light scheme
        Cell::new(
            format_argb_as_rgb([
                color_light.alpha,
                color_light.red,
                color_light.green,
                color_light.blue,
            ])
            .to_uppercase()
            .as_str(),
        )
        .style_spec("c"),
        Cell::new(format!("{}", formatstr.style(generate_style(&color_light))).as_str())
            .style_spec("c"),
        // Dark scheme
        Cell::new(
            format_argb_as_rgb([
                color_dark.alpha,
                color_dark.red,
                color_dark.green,
                color_dark.blue,
            ])
            .to_uppercase()
            .as_str(),
        )
        .style_spec("c"),
        Cell::new(format!("{}", formatstr.style(generate_style(&color_dark))).as_str())
            .style_spec("c"),
    ]));
}

fn generate_style(color: &Color) -> Style {
    let luma = color.red as u16 + color.blue as u16 + color.green as u16;

    let owo_color: owo_colors::Rgb = owo_colors::Rgb(color.red, color.green, color.blue);

    if luma > 500 {
        Style::new().black().on_color(owo_color)
    } else {
        Style::new().white().on_color(owo_color)
    }
}

pub fn get_source_color(source: &Source) -> Result<[u8; 4], Report> {
    let source_color: [u8; 4] = match &source {
        Source::Image { path } => {
            // test
            info!("Opening image in <d><u>{}</>", path);
            let img = match image::open(path) {
                Ok(img) => img,
                Err(ImageError::Unsupported(e)) => {
                    return Err(Report::new(e).suggestion("Try using another image that is valid."))
                }
                Err(ImageError::IoError(e)) => {
                    return Err(Report::new(e).suggestion(
                        "Try using an image that exists or make sure the path provided is valid.",
                    ))
                }
                Err(e) => return Err(Report::new(e)),
            };
            source_color_from_image(img)?[0]
        }
        Source::WebImage { url } => {
            // test
            info!("Fetching image from <d><u>{}</>", url);

            let img = fetch_image(url)?;
            source_color_from_image(img)?[0]
        }
        Source::Color(color) => {
            let src: Rgb = match color {
                ColorFormat::Hex { string } => {
                    Rgb::from_hex_str(string).expect("Invalid hex color string provided")
                }
                ColorFormat::Rgb { string } => {
                    string.parse().expect("Invalid rgb color string provided")
                }
                ColorFormat::Hsl { string } => Hsl::from_str(string)
                    .expect("Invalid hsl color string provided")
                    .into(),
            };
            [
                src.alpha() as u8,
                src.red() as u8,
                src.green() as u8,
                src.blue() as u8,
            ]
        }
    };
    Ok(source_color)
}
