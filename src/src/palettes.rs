use crate::colors::Color;
use std::collections::{HashMap, VecDeque};

pub struct Palette {
    colors: Vec<Color>,
}

impl Palette {
    pub fn get_stroke_color(&self) -> Color {
        Color::black()
    }

    pub fn get_default_fill_color(&self, index: usize) -> Color {
        Color::white()
    }

    pub fn get_fill_color(&self, index: usize) -> Color {
        if self.colors.is_empty() {
            return Color::white();
        }

        let index = index % self.colors.len();
        self.colors[index]
    }
}

pub struct PaletteReader {}

impl PaletteReader {
    pub fn read(&self, content: &str) -> Result<HashMap<String, Palette>, anyhow::Error> {
        let mut result = HashMap::new();

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
            result.insert(name.to_string(), palette);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_single_palette() {
        let content = "my_palette: #00ffff #ff0000 #ffff00";
        let reader = PaletteReader {};
        let palettes = reader.read(content).unwrap();
        assert_eq!(palettes.len(), 1);
        assert!(palettes.contains_key("my_palette"));
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
        assert_eq!(palettes.len(), 7);
    }
}
