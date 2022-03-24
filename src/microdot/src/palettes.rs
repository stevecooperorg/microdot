use crate::colors::{Color, Colors};
use palette::{FromColor, Hue, IntoColor, Lch, Srgb};
use std::collections::{HashMap, VecDeque};

#[derive(Clone)]
pub struct Palette {
    colors: Vec<Color>,
}

impl Palette {
    pub fn get_stroke_color(&self) -> Color {
        Colors::black()
    }

    pub fn get_fill_color(&self, index: usize) -> Color {
        if self.colors.is_empty() {
            return Colors::white();
        }

        let index = index % self.colors.len();
        self.colors[index]
    }
}

pub struct PaletteReader {}

pub struct PaletteCollection {
    inner: HashMap<String, Palette>,
}

impl PaletteCollection {
    fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    fn insert(&mut self, name: &str, palette: Palette) {
        self.inner.insert(name.to_string(), palette);
    }

    pub fn get(&self, name: &str) -> Option<Palette> {
        self.inner.get(name).map(|p| p.clone())
    }
}

impl PaletteReader {
    pub fn read(&self, content: &str) -> Result<PaletteCollection, anyhow::Error> {
        let mut result = PaletteCollection::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if line.starts_with('#') {
                continue;
            }

            let mut first_split: VecDeque<_> = line.split(':').collect();
            if first_split.len() != 2 {
                return Err(anyhow::Error::msg(
                    "expected two parts separated by a colon",
                ));
            }

            let name = first_split.pop_front().unwrap().trim();
            let color_list = first_split.pop_front().unwrap().trim();
            let color_list: Vec<_> = color_list.split(' ').map(|w| w.trim()).collect();

            let mut colors = vec![];

            for color_str in color_list {
                let color = Color::from_html_string(color_str)?;
                colors.push(color);
            }

            let palette = Palette { colors };
            result.insert(name, palette);
        }

        let generator = ColorIterator::new();
        let colors: Vec<_> = generator.take(20).collect();
        let palette = Palette { colors };
        result.insert("generated", palette);
        Ok(result)
    }
}

pub struct ColorIterator {
    hue: f32,
    iteration: usize,
}

impl ColorIterator {
    fn new() -> Self {
        Self {
            hue: 0f32,
            iteration: 1,
        }
    }
}

impl Iterator for ColorIterator {
    type Item = Color;

    fn next(&mut self) -> Option<Self::Item> {
        let delta = 140.0f32;
        let hsl: Lch = Srgb::new(1.0, 0.5, 0.5).into_color();
        let hsl = hsl.shift_hue(self.hue);
        self.hue += delta;
        let rgb: Srgb = Srgb::from_color(hsl);
        fn to255(component: f32) -> u8 {
            let res = component * 256.0;
            let res = f32::min(res, 256.0);
            let res = f32::max(res, 0.0);
            let res = res as u8;
            res
        }

        let r = to255(rgb.red);
        let g = to255(rgb.green);
        let b = to255(rgb.blue);
        Some(Color::from_rgb(r, g, b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_iterator_generates_colors() {
        let iter = ColorIterator::new();
        let colors: Vec<_> = iter.take(20).collect();
        let color_str = colors
            .iter()
            .map(Color::to_html_string)
            .collect::<Vec<_>>()
            .join(" ");

        let palette = format!("generated: {}", color_str);
        println!("{}", palette);
    }

    #[test]
    fn can_read_single_palette() {
        let content = "my_palette: #00ffff #ff0000 #ffff00";
        let reader = PaletteReader {};
        let palettes = reader.read(content).unwrap();
        let palette = palettes.get("my_palette").unwrap();
        assert_eq!(Color::from_rgb(0, 255, 255), palette.get_fill_color(0));
        assert_eq!(Color::from_rgb(255, 0, 0), palette.get_fill_color(1));
        assert_eq!(Color::from_rgb(255, 255, 0), palette.get_fill_color(2));
        assert_eq!(Color::from_rgb(0, 255, 255), palette.get_fill_color(3));
    }

    #[test]
    fn can_read_palette_file() {
        let content = include_str!("./palettes.txt");
        let reader = PaletteReader {};
        let palettes = reader.read(content).unwrap();
        assert!(palettes.get("antarctica_evening").is_some());
    }
}
