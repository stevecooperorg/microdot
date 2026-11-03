//! Day / night colour schemes for the TUI.

use microdot_termgraph::{EdgeStyle, NodeStyle};
use ratatui::style::Color;

/// Named palette for the full-screen editor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThemeKind {
    /// Light background terminal: dark ink, muted greys, vivid accents.
    Day,
    /// Dark background terminal (default): light ink, soft greys, vivid accents.
    Night,
}

/// Colours used by chrome (lists, menu, footer) and the graph preview.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub kind: ThemeKind,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub status: Color,
    pub menu_fg: Color,
    pub menu_bg: Color,
    pub menu_sel_fg: Color,
    pub menu_sel_bg: Color,
    pub focus_border: Color,
    pub node_sel_fg: Color,
    pub node_sel_bg: Color,
    pub edge_sel_fg: Color,
    pub edge_sel_bg: Color,
    pub search_hit: Color,
    pub node_border: Color,
    pub node_text: Color,
    pub edge_line: Color,
    pub edge_label: Color,
    pub selected_node_border: Color,
    pub selected_node_text: Color,
    pub selected_edge: Color,
    pub search_node_border: Color,
    pub search_node_text: Color,
}

impl Theme {
    pub fn day() -> Self {
        Self {
            kind: ThemeKind::Day,
            text: Color::Black,
            muted: Color::DarkGray,
            accent: Color::Blue,
            status: Color::Blue,
            menu_fg: Color::Black,
            menu_bg: Color::Reset,
            menu_sel_fg: Color::White,
            menu_sel_bg: Color::Blue,
            focus_border: Color::Blue,
            node_sel_fg: Color::White,
            node_sel_bg: Color::Green,
            edge_sel_fg: Color::Black,
            edge_sel_bg: Color::Yellow,
            search_hit: Color::Magenta,
            node_border: Color::Black,
            node_text: Color::Black,
            edge_line: Color::DarkGray,
            edge_label: Color::DarkGray,
            selected_node_border: Color::Green,
            selected_node_text: Color::Black,
            selected_edge: Color::Rgb(180, 100, 0),
            search_node_border: Color::Magenta,
            search_node_text: Color::DarkGray,
        }
    }

    pub fn night() -> Self {
        Self {
            kind: ThemeKind::Night,
            text: Color::White,
            muted: Color::DarkGray,
            accent: Color::Cyan,
            status: Color::Cyan,
            menu_fg: Color::White,
            menu_bg: Color::Reset,
            menu_sel_fg: Color::Black,
            menu_sel_bg: Color::Cyan,
            focus_border: Color::Cyan,
            node_sel_fg: Color::Black,
            node_sel_bg: Color::Green,
            edge_sel_fg: Color::Black,
            edge_sel_bg: Color::Yellow,
            search_hit: Color::DarkGray,
            node_border: Color::White,
            node_text: Color::White,
            edge_line: Color::Gray,
            edge_label: Color::DarkGray,
            selected_node_border: Color::Green,
            selected_node_text: Color::White,
            selected_edge: Color::Yellow,
            search_node_border: Color::DarkGray,
            search_node_text: Color::Gray,
        }
    }

    pub fn default_node_style(self) -> NodeStyle {
        NodeStyle {
            border: microdot_termgraph::BoxBorder::Rounded,
            border_color: self.node_border,
            text_color: self.node_text,
        }
    }

    pub fn default_edge_style(self) -> EdgeStyle {
        EdgeStyle {
            line_color: self.edge_line,
            text_color: self.edge_label,
        }
    }
}
