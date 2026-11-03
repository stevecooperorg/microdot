//! Character grid for terminal rendering.
//!
//! Adapted from ascii-petgraph (MIT).

use ratatui::style::Color;

/// A character cell in the render grid.
#[derive(Debug, Clone)]
pub struct Cell {
    pub char: char,
    pub fg: Color,
    pub bg: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            char: ' ',
            fg: Color::White,
            bg: Color::Reset,
        }
    }
}

/// A 2D grid of characters for rendering.
pub struct CharGrid {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

impl CharGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![Cell::default(); width.saturating_mul(height).max(1)],
            width: width.max(1),
            height: height.max(1),
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = cell;
        }
    }

    pub fn set_char(&mut self, x: usize, y: usize, c: char, fg: Color) {
        self.set(
            x,
            y,
            Cell {
                char: c,
                fg,
                bg: Color::Reset,
            },
        );
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        if x < self.width && y < self.height {
            Some(&self.cells[y * self.width + x])
        } else {
            None
        }
    }

    pub fn draw_hline(&mut self, x: usize, y: usize, len: usize, c: char, fg: Color) {
        for i in 0..len {
            self.set_char(x + i, y, c, fg);
        }
    }

    pub fn draw_vline(&mut self, x: usize, y: usize, len: usize, c: char, fg: Color) {
        for i in 0..len {
            self.set_char(x, y + i, c, fg);
        }
    }

    pub fn draw_text(&mut self, x: usize, y: usize, text: &str, fg: Color) {
        for (i, c) in text.chars().enumerate() {
            self.set_char(x + i, y, c, fg);
        }
    }

    pub fn to_string_plain(&self) -> String {
        let mut output = String::with_capacity((self.width + 1) * self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(cell) = self.get(x, y) {
                    output.push(cell.char);
                }
            }
            output.push('\n');
        }
        output
    }
}
