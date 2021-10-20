use crate::palettes::PaletteReader;
use std::cmp::Ordering;

#[derive(serde::Deserialize, Copy, Clone, PartialEq, Debug)]
pub struct Color([u8; 3]);

impl PartialOrd for Color {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.sort_key().cmp(&other.sort_key()))
    }
}

impl Color {
    fn r(&self) -> u8 {
        self.0[0]
    }

    fn g(&self) -> u8 {
        self.0[1]
    }

    fn b(&self) -> u8 {
        self.0[2]
    }

    fn mix(&self, other: &Self, fraction: f64) -> Self {
        let r = Color::blend(self.r(), other.r(), fraction);
        let g = Color::blend(self.g(), other.g(), fraction);
        let b = Color::blend(self.b(), other.b(), fraction);
        Color([r, g, b])
    }

    pub fn darken(&self, fraction: f64) -> Self {
        self.mix(&Color::black(), fraction)
    }

    pub fn lighten(&self, fraction: f64) -> Self {
        self.mix(&Color::white(), fraction)
    }

    pub fn black() -> Self {
        Color([0, 0, 0])
    }

    pub fn monochrome(&self) -> Self {
        let grey =
            ((self.r() as u64 + self.g() as u64 + self.b() as u64) as f64 / 3.0f64).floor() as u8;
        Color::from_rgb(grey, grey, grey)
    }

    pub fn desaturate(&self, fraction: f64) -> Self {
        self.mix(&self.monochrome(), fraction)
    }

    pub fn white() -> Self {
        Color([255, 255, 255])
    }

    fn blend(c1: u8, c2: u8, fraction: f64) -> u8 {
        let r = c1 as f64 * fraction + c2 as f64 * (1.0f64 - fraction);
        r.floor() as u8
    }

    pub fn to_html_string(&self) -> String {
        fn push_hex(s: &mut String, byte: u8) {
            use std::fmt::Write;
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

    fn sort_key(&self) -> i64 {
        -((self.r() as f64 * 1.0f64)
            + (self.g() as f64 * 1.00001f64)
            + (self.b() as f64 * 1.001f64)) as i64
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color([r, g, b])
    }
}

pub struct ColorScheme {
    font_color: Color,
    fill_color: Color,
    stroke_color: Color,
    node_border_width: f64,
}

const PALETTE_NAME: &str = "antarctica_evening_v2";

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
        ColorScheme::from_colors(Color::black(), Color::white(), Color::black())
    }

    pub fn series(highlight: usize) -> Self {
        ColorScheme::from_entry(2)
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
    fn does_not_parse_non_html_colors() {
        let naughty_strings = ["", "nope", "seven..", "#00000g"];

        for naughty in &naughty_strings {
            assert!(Color::from_html_string(naughty).is_err());
        }
    }
}
