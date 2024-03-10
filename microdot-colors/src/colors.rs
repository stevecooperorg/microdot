use crate::palettes::PaletteReader;
use palette::*;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Color {
    inner: Srgb<u8>,
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_html_string())
    }
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color {
            inner: Srgb::new(r, g, b),
        }
    }

    pub fn from_hsl(h: f64, s: f64, l: f64) -> Self {
        let s = s / 100.0;
        let l = l / 100.0;

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
        let m = l - c / 2.0;

        let (r_prime, g_prime, b_prime) = if (0.0..60.0).contains(&h) {
            (c, x, 0.0)
        } else if (60.0..120.0).contains(&h) {
            (x, c, 0.0)
        } else if (120.0..180.0).contains(&h) {
            (0.0, c, x)
        } else if (180.0..240.0).contains(&h) {
            (0.0, x, c)
        } else if (240.0..300.0).contains(&h) {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        let r = ((r_prime + m) * 255.0).round() as u8;
        let g = ((g_prime + m) * 255.0).round() as u8;
        let b = ((b_prime + m) * 255.0).round() as u8;

        Self::from_rgb(r, g, b)
    }

    pub fn to_rgb(&self) -> (u8, u8, u8) {
        let r = self.r();
        let g = self.g();
        let b = self.b();
        (r, g, b)
    }
    pub fn to_hsl(&self) -> (f64, f64, f64) {
        let r = self.r() as f64 / 255.0;
        let g = self.g() as f64 / 255.0;
        let b = self.b() as f64 / 255.0;

        let max = r.max(g.max(b));
        let min = r.min(g.min(b));

        let mut h = 0.0;
        let mut s = 0.0;
        let l = (max + min) / 2.0;

        if max != min {
            let d = max - min;
            s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };

            h = if max == r {
                (g - b) / d + (if g < b { 6.0 } else { 0.0 })
            } else if max == g {
                (b - r) / d + 2.0
            } else {
                (r - g) / d + 4.0
            };

            h /= 6.0;
        }

        (h * 360.0, s * 100.0, l * 100.0)
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

    pub fn mix(&self, other: Color) -> Color {
        self.interpolate(other, 0.5)
    }

    pub fn mute(&self, saturation_decrease: f64, lightness_adjustment: f64) -> Self {
        let (h, s, l) = self.to_hsl(); // Convert RGB to HSL

        // Calculate new saturation, ensuring it doesn't go below 0 or above 100
        let new_s = (s * saturation_decrease).max(0.0).min(100.0);

        // Adjust lightness, ensuring it stays within bounds
        let new_l = (l * lightness_adjustment).max(0.0).min(100.0);

        // Convert back to RGB and return the new color
        Color::from_hsl(h, new_s, new_l)
    }

    pub fn interpolate(&self, color2: Color, normalised_position_through_part: f64) -> Color {
        let (r, g, b) = self.to_rgb();
        let (r2, g2, b2) = color2.to_rgb();
        let r = (r as f64 * (1.0 - normalised_position_through_part)
            + r2 as f64 * normalised_position_through_part) as u8;
        let g = (g as f64 * (1.0 - normalised_position_through_part)
            + g2 as f64 * normalised_position_through_part) as u8;
        let b = (b as f64 * (1.0 - normalised_position_through_part)
            + b2 as f64 * normalised_position_through_part) as u8;
        Color::from_rgb(r, g, b)
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
// const PALETTE_NAME: &str = "generated";
// const PALETTE_NAME: &str = "large";
const PALETTE_NAME: &str = "stretched";

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
    fn it_can_mix_colors() {
        let actual = Color::from_rgb(255, 0, 0).mix(Color::from_rgb(0, 255, 0));
        let expected = Color::from_rgb(127, 127, 0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn does_not_parse_non_html_colors() {
        let naughty_strings = ["", "nope", "seven..", "#00000g"];
        for naughty in &naughty_strings {
            assert!(Color::from_html_string(naughty).is_err());
        }
    }

    #[test]
    fn can_mute_colors() {
        let actual = Color::from_rgb(255, 0, 0).mute(0.5, 1.0);
        let expected = Color::from_rgb(191, 64, 64);
        assert_eq!(actual, expected, "desaturation");

        let actual = Color::from_rgb(255, 0, 0).mute(1.0, 0.5);
        let expected = Color::from_rgb(128, 0, 0);
        assert_eq!(actual, expected, "darken");
    }

    #[test]
    fn can_convert_to_and_from_hsl() {
        let color = Color::from_rgb(255, 0, 0);
        let (h, s, l) = color.to_hsl();
        assert_eq!(h, 0.0);
        assert_eq!(s, 100.0);
        assert_eq!(l, 50.0);
        let actual = Color::from_hsl(h, s, l);
        assert_eq!(actual, color);
    }
}
