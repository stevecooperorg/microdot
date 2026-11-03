//! Draw a [`Scene`] onto a [`CharGrid`].
//!
//! Edge routing adapted from ascii-petgraph (MIT).

use crate::grid::CharGrid;
use crate::scene::{LabelMode, PlacedEdge, PlacedNode, Scene};
use crate::style::EdgeChars;

/// Width of a node box for a display label (borders + padding).
pub fn node_box_width(display_label: &str) -> usize {
    display_label.chars().count() + 4
}

pub const NODE_BOX_HEIGHT: usize = 3;

pub fn display_label(node: &PlacedNode, mode: LabelMode) -> String {
    match mode {
        LabelMode::Full => node.label.clone(),
        LabelMode::Truncate(max_len) => {
            let s = &node.label;
            let count = s.chars().count();
            if count <= max_len {
                s.clone()
            } else {
                let keep = max_len.saturating_sub(1);
                format!("{}…", s.chars().take(keep).collect::<String>())
            }
        }
        LabelMode::Ids => node.id.clone(),
    }
}

/// Render the scene to a character grid.
pub fn render_scene(scene: &Scene) -> CharGrid {
    let nodes: Vec<PreparedNode> = scene
        .nodes
        .iter()
        .map(|n| {
            let label = display_label(n, scene.label_mode);
            let width = node_box_width(&label);
            PreparedNode {
                id: n.id.clone(),
                label,
                x: n.x,
                y: n.y,
                width,
                height: NODE_BOX_HEIGHT,
                style: n.style.clone(),
            }
        })
        .collect();

    let pad = scene.padding;
    let max_x = nodes
        .iter()
        .map(|n| n.x + n.width)
        .max()
        .unwrap_or(0)
        .saturating_add(pad)
        .max(1);
    let max_y = nodes
        .iter()
        .map(|n| n.y + n.height)
        .max()
        .unwrap_or(0)
        .saturating_add(pad)
        .max(1);

    let mut grid = CharGrid::new(max_x, max_y);
    let edge_chars = EdgeChars::default();

    for edge in &scene.edges {
        render_edge(&mut grid, edge, &nodes, &edge_chars);
    }
    for node in &nodes {
        render_node(&mut grid, node);
    }

    grid
}

struct PreparedNode {
    id: String,
    label: String,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    style: crate::style::NodeStyle,
}

impl PreparedNode {
    fn center_x(&self) -> usize {
        self.x + self.width / 2
    }
    fn bottom_y(&self) -> usize {
        self.y + self.height - 1
    }
    fn top_y(&self) -> usize {
        self.y
    }
}

fn render_node(grid: &mut CharGrid, node: &PreparedNode) {
    let chars = node.style.border.chars();
    let width = node.width;

    grid.set_char(node.x, node.y, chars.top_left, node.style.border_color);
    grid.draw_hline(
        node.x + 1,
        node.y,
        width.saturating_sub(2),
        chars.horizontal,
        node.style.border_color,
    );
    grid.set_char(
        node.x + width - 1,
        node.y,
        chars.top_right,
        node.style.border_color,
    );

    let mid_y = node.y + 1;
    grid.set_char(node.x, mid_y, chars.vertical, node.style.border_color);
    grid.set_char(node.x + 1, mid_y, ' ', node.style.text_color);
    grid.draw_text(node.x + 2, mid_y, &node.label, node.style.text_color);
    grid.set_char(node.x + width - 2, mid_y, ' ', node.style.text_color);
    grid.set_char(
        node.x + width - 1,
        mid_y,
        chars.vertical,
        node.style.border_color,
    );

    let bot_y = node.y + 2;
    grid.set_char(node.x, bot_y, chars.bottom_left, node.style.border_color);
    grid.draw_hline(
        node.x + 1,
        bot_y,
        width.saturating_sub(2),
        chars.horizontal,
        node.style.border_color,
    );
    grid.set_char(
        node.x + width - 1,
        bot_y,
        chars.bottom_right,
        node.style.border_color,
    );
}

fn render_edge(
    grid: &mut CharGrid,
    edge: &PlacedEdge,
    nodes: &[PreparedNode],
    chars: &EdgeChars,
) {
    let color = edge.style.line_color;
    let label = &edge.label;
    let offset = edge.parallel_offset;

    let source = nodes.iter().find(|n| n.id == edge.from);
    let target = nodes.iter().find(|n| n.id == edge.to);
    let (source, target) = match (source, target) {
        (Some(s), Some(t)) => (s, t),
        _ => return,
    };

    let apply_offset = |x: usize| -> usize {
        if offset >= 0 {
            x.saturating_add(offset as usize)
        } else {
            x.saturating_sub((-offset) as usize)
        }
    };

    let source_bottom = source.bottom_y();
    let source_top = source.top_y();
    let target_bottom = target.bottom_y();
    let target_top = target.top_y();

    if source_bottom < target_top {
        let start_x = apply_offset(source.center_x());
        let start_y = source_bottom + 1;
        let end_x = apply_offset(target.center_x());
        let end_y = target_top.saturating_sub(1);
        render_vertical_edge(
            grid,
            chars,
            color,
            start_x,
            start_y,
            end_x,
            end_y,
            true,
            label,
            edge.style.text_color,
            offset,
        );
    } else if target_bottom < source_top {
        let start_x = apply_offset(source.center_x());
        let start_y = source_top.saturating_sub(1);
        let end_x = apply_offset(target.center_x());
        let end_y = target_bottom + 1;
        render_vertical_edge(
            grid,
            chars,
            color,
            start_x,
            start_y,
            end_x,
            end_y,
            false,
            label,
            edge.style.text_color,
            offset,
        );
    } else {
        let (start_x, end_x) = if source.x + source.width < target.x {
            (source.x + source.width, target.x.saturating_sub(1))
        } else {
            (source.x.saturating_sub(1), target.x + target.width)
        };
        let base_y = source.y + source.height / 2;
        let y = if offset >= 0 {
            base_y.saturating_add(offset as usize)
        } else {
            base_y.saturating_sub((-offset) as usize)
        };
        let (left, right) = if start_x < end_x {
            (start_x, end_x)
        } else {
            (end_x, start_x)
        };
        for x in left..=right {
            grid.set_char(x, y, chars.horizontal, color);
        }
        let arrow = if end_x > start_x {
            chars.arrow_right
        } else {
            chars.arrow_left
        };
        grid.set_char(end_x, y, arrow, color);

        if !label.is_empty() {
            let edge_len = right.saturating_sub(left);
            let label_x = if edge_len > label.chars().count() + 2 {
                left + (edge_len - label.chars().count()) / 2
            } else {
                left
            };
            let label_y = if offset <= 0 {
                y.saturating_sub(1)
            } else {
                y + 1
            };
            grid.draw_text(label_x, label_y, label, edge.style.text_color);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_vertical_edge(
    grid: &mut CharGrid,
    chars: &EdgeChars,
    color: ratatui::style::Color,
    start_x: usize,
    start_y: usize,
    end_x: usize,
    end_y: usize,
    going_down: bool,
    label: &str,
    label_color: ratatui::style::Color,
    parallel_offset: i32,
) {
    if start_x == end_x {
        let (y_start, y_end) = if start_y < end_y {
            (start_y, end_y)
        } else {
            (end_y, start_y)
        };
        for y in y_start..y_end {
            grid.set_char(start_x, y, chars.vertical, color);
        }
        let arrow = if going_down {
            chars.arrow_down
        } else {
            chars.arrow_up
        };
        grid.set_char(end_x, end_y, arrow, color);

        if !label.is_empty() {
            let label_y = (y_start + y_end) / 2;
            if parallel_offset < 0 {
                let label_x = start_x.saturating_sub(label.chars().count() + 1);
                grid.draw_text(label_x, label_y, label, label_color);
            } else {
                grid.draw_text(start_x + 1, label_y, label, label_color);
            }
        }
    } else {
        let mid_y = (start_y + end_y) / 2;
        let (y_min, y_max) = if start_y < end_y {
            (start_y, end_y)
        } else {
            (end_y, start_y)
        };
        let mid_y = mid_y.max(y_min).min(y_max);

        let (v1_start, v1_end) = if start_y < mid_y {
            (start_y, mid_y)
        } else {
            (mid_y, start_y)
        };
        for y in v1_start..v1_end {
            grid.set_char(start_x, y, chars.vertical, color);
        }

        let corner1 = if end_x > start_x {
            if going_down {
                chars.corner_up_right
            } else {
                chars.corner_down_right
            }
        } else if going_down {
            chars.corner_up_left
        } else {
            chars.corner_down_left
        };
        grid.set_char(start_x, mid_y, corner1, color);

        let (left_x, right_x) = if start_x < end_x {
            (start_x + 1, end_x)
        } else {
            (end_x + 1, start_x)
        };
        for x in left_x..right_x {
            grid.set_char(x, mid_y, chars.horizontal, color);
        }

        let corner2 = if end_x > start_x {
            if going_down {
                chars.corner_down_left
            } else {
                chars.corner_up_left
            }
        } else if going_down {
            chars.corner_down_right
        } else {
            chars.corner_up_right
        };
        grid.set_char(end_x, mid_y, corner2, color);

        let (v2_start, v2_end) = if mid_y < end_y {
            (mid_y + 1, end_y)
        } else {
            (end_y + 1, mid_y)
        };
        for y in v2_start..v2_end {
            grid.set_char(end_x, y, chars.vertical, color);
        }

        let arrow = if going_down {
            chars.arrow_down
        } else {
            chars.arrow_up
        };
        grid.set_char(end_x, end_y, arrow, color);

        if !label.is_empty() {
            let v1_len = v1_end.saturating_sub(v1_start);
            let h_len = right_x.saturating_sub(left_x);
            if v1_len >= 2 {
                let label_y = (v1_start + v1_end) / 2;
                if parallel_offset < 0 {
                    let label_x = start_x.saturating_sub(label.chars().count() + 1);
                    grid.draw_text(label_x, label_y, label, label_color);
                } else {
                    grid.draw_text(start_x + 1, label_y, label, label_color);
                }
            } else if h_len > label.chars().count() + 2 {
                let label_x = ((left_x + right_x) / 2).saturating_sub(label.chars().count() / 2);
                let label_y = if parallel_offset < 0 {
                    mid_y.saturating_sub(1)
                } else {
                    mid_y + 1
                };
                grid.draw_text(label_x, label_y, label, label_color);
            } else {
                let label_y = (v2_start + v2_end) / 2;
                if parallel_offset < 0 {
                    let label_x = end_x.saturating_sub(label.chars().count() + 1);
                    grid.draw_text(label_x, label_y, label, label_color);
                } else {
                    grid.draw_text(end_x + 1, label_y, label, label_color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{PlacedEdge, PlacedNode, Scene};
    use crate::style::{EdgeStyle, NodeStyle};

    #[test]
    fn renders_two_nodes_and_edge() {
        let mut scene = Scene::new();
        scene.nodes = vec![
            PlacedNode {
                id: "n0".into(),
                label: "A".into(),
                x: 2,
                y: 1,
                style: NodeStyle::default(),
            },
            PlacedNode {
                id: "n1".into(),
                label: "B".into(),
                x: 2,
                y: 8,
                style: NodeStyle::default(),
            },
        ];
        scene.edges = vec![PlacedEdge {
            id: "e0".into(),
            from: "n0".into(),
            to: "n1".into(),
            label: String::new(),
            style: EdgeStyle::default(),
            parallel_offset: 0,
        }];
        let grid = render_scene(&scene);
        let text = grid.to_string_plain();
        assert!(text.contains('A'));
        assert!(text.contains('B'));
        assert!(text.contains('↓') || text.contains('│'));
    }
}
