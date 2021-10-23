use crate::palettes::PaletteReader;
use palette::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    inner: Srgb<u8>,
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color {
            inner: Srgb::new(r, g, b),
        }
    }

    fn r(&self) -> u8 {
        self.inner.red
    }

    fn g(&self) -> u8 {
        self.inner.green
    }

    fn b(&self) -> u8 {
        self.inner.blue
    }

    pub fn to_html_string(&self) -> String {
        fn push_hex(s: &mut String, byte: u8) {
            use std::fmt::Write;
            if byte < 16 {
                write!(s, "0").expect("unable to write");
            }
            write!(s, "{:X}", byte).expect("Unable to write");
        }

        let mut s = String::new();
        push_hex(&mut s, self.r());
        push_hex(&mut s, self.g());
        push_hex(&mut s, self.b());

        format!("#{}", s)
    }

    pub fn from_html_string(html_str: &str) -> Result<Color, anyhow::Error> {
        if html_str.len() != 7 {
            return Err(anyhow::Error::msg(
                "expected seven characters in an html color code",
            ));
        }

        if !html_str.starts_with('#') {
            return Err(anyhow::Error::msg(
                "first character must be '#' in an html color code",
            ));
        }

        let bytes = &html_str[1..7];
        let decoded = hex::decode(bytes)
            .map_err(|_| anyhow::Error::msg("Decoding from hex failed in an html color code"))?;

        let color = Color::from_rgb(decoded[0], decoded[1], decoded[2]);

        Ok(color)
    }
}

pub struct Colors;

impl Colors {
    pub fn black() -> Color {
        Color::from_rgb(0, 0, 0)
    }
    pub fn white() -> Color {
        Color::from_rgb(255, 255, 255)
    }
}

pub struct ColorScheme {
    font_color: Color,
    fill_color: Color,
    stroke_color: Color,
    node_border_width: f64,
}

// const PALETTE_NAME: &str = "antarctica_evening_v2";
const PALETTE_NAME: &str = "generated";

impl ColorScheme {
    const NODE_BORDER_WIDTH: f64 = 3.0f64;

    fn from_colors(stroke_color: Color, fill_color: Color, font_color: Color) -> Self {
        Self {
            font_color,
            fill_color,
            stroke_color,
            node_border_width: Self::NODE_BORDER_WIDTH,
        }
    }

    fn from_entry(i: usize) -> Self {
        let content = include_str!("./palettes.txt");
        let reader = PaletteReader {};
        let palettes = reader.read(content).expect("couldn't read palette");
        let palette = palettes.get(PALETTE_NAME).unwrap();

        let stroke_color = palette.get_stroke_color();
        let fill_color = palette.get_fill_color(i);
        let font_color = stroke_color;
        Self::from_colors(stroke_color, fill_color, font_color)
    }

    pub fn normal() -> Self {
        ColorScheme::from_colors(Colors::black(), Colors::white(), Colors::black())
    }

    pub fn series(highlight: usize) -> Self {
        ColorScheme::from_entry(highlight)
    }

    pub fn get_stroke_color(&self) -> Color {
        self.stroke_color
    }
    pub fn get_font_color(&self) -> Color {
        self.font_color
    }
    pub fn get_fill_color(&self) -> Color {
        self.fill_color
    }
    pub fn get_node_border_width(&self) -> f64 {
        self.node_border_width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_html_color_codes() {
        let actual = Color::from_html_string("#ff1100").expect("could not parse");
        let expected = Color::from_rgb(255, 17, 0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_can_print_html_colors() {
        assert_eq!("#000000", &Colors::black().to_html_string());
        assert_eq!("#FFFFFF", &Colors::white().to_html_string());
        assert_eq!("#FF0000", &Color::from_rgb(255, 0, 0).to_html_string());
    }

    #[test]
    fn does_not_parse_non_html_colors() {
        let naughty_strings = ["", "nope", "seven..", "#00000g"];
        for naughty in &naughty_strings {
            assert!(Color::from_html_string(naughty).is_err());
        }
    }
}
