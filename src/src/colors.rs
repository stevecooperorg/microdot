#[derive(serde::Deserialize, Copy, Clone)]
pub struct Color([u8; 3]);

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
}

#[derive(serde::Deserialize)]
pub struct KhromaData {
    favorites: Vec<Favorite>,
}

impl KhromaData {
    pub fn entry(&self, i: usize) -> [Color; 2] {
        let i = i % self.favorites.len();
        self.favorites.get(i).unwrap().colors.clone()
    }

    pub fn new() -> KhromaData {
        let json = include_str!("./my_khroma_data.json");
        serde_json::from_str(json).expect("bad data")
    }
}

#[derive(serde::Deserialize)]
struct Favorite {
    id: String,
    colors: [Color; 2],
}

#[derive(serde::Deserialize)]
struct JsonPalette {
    result: [Color; 5],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_khroma_data() {
        let data = KhromaData::new();
        assert_eq!(data.favorites.len(), 90);
    }
}
