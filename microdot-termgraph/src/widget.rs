//! Ratatui widget for a [`Scene`].

use crate::render::render_scene;
use crate::scene::Scene;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

/// Draws a positioned [`Scene`] into a terminal area.
pub struct SceneWidget<'a> {
    pub scene: &'a Scene,
}

impl Widget for SceneWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let grid = render_scene(self.scene);
        let (gw, gh) = grid.size();
        let max_x = (area.width as usize).min(gw);
        let max_y = (area.height as usize).min(gh);
        for y in 0..max_y {
            for x in 0..max_x {
                if let Some(cell) = grid.get(x, y) {
                    if let Some(term_cell) = buf.cell_mut((area.x + x as u16, area.y + y as u16)) {
                        term_cell.set_char(cell.char);
                        term_cell.set_style(Style::default().fg(cell.fg).bg(Color::Reset));
                    }
                }
            }
        }
    }
}
