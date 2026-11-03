//! Layout and drawing for the microdot TUI.

use super::app::{App, ConfirmKind, Focus};
use super::menu::MenuGroup;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

const HELP_TEXT: &str = "\
Microdot TUI keys

Navigation
  Tab          cycle Nodes / Edges / Graph
  j k / arrows move in focused pane
  J K          jump ±10
  / .          search / next match
  `            menu
  :            command (REPL language)
  ?            this help
  q            quit

Edit (current selection)
  i            insert node
  a / b        insert after / before
  r            rename
  x / X        delete / delete keep edges
  l            link to another node
  u            unlink current edge
  e            expand current edge

Esc cancels drafts, menus, and confirms.
";

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let footer_h = if app.draft.is_some() { 4 } else { 3 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(footer_h),
        ])
        .split(area);

    draw_menubar(frame, app, chunks[0]);
    draw_main(frame, app, chunks[1]);
    draw_footer(frame, app, chunks[2]);

    if app.menu_open {
        draw_menu_dropdown(frame, app, chunks[0], chunks[1]);
    }
    if app.show_help {
        draw_help(frame, area);
    }
}

fn draw_menubar(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();
    for (i, group) in MenuGroup::ALL.iter().enumerate() {
        let selected = app.menu_open && app.menu_group == i;
        let style = if selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        spans.push(Span::styled(format!(" {} ", group.label()), style));
    }
    if !app.menu_open {
        spans.push(Span::styled(
            "  [`] menu",
            Style::default().fg(Color::DarkGray),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_menu_dropdown(frame: &mut Frame, app: &App, menubar: Rect, body: Rect) {
    let items = app.active_menu_items();
    let width = items
        .iter()
        .map(|a| a.label().chars().count())
        .max()
        .unwrap_or(12) as u16
        + 4;
    let height = items.len() as u16 + 2;
    let mut x = menubar.x;
    for i in 0..app.menu_group {
        x += MenuGroup::ALL[i].label().len() as u16 + 2;
    }
    let area = Rect {
        x: x.min(body.x + body.width.saturating_sub(width)),
        y: body.y,
        width: width.min(body.width),
        height: height.min(body.height),
    };
    frame.render_widget(Clear, area);
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let style = if i == app.menu_item {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(format!(" {} ", action.label()), style))
        })
        .collect();
    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(MenuGroup::ALL[app.menu_group].label()),
    );
    frame.render_widget(list, area);
}

fn draw_main(frame: &mut Frame, app: &mut App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(36), Constraint::Min(20)])
        .split(area);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(cols[0]);

    draw_nodes(frame, app, left[0]);
    draw_edges(frame, app, left[1]);
    draw_graph(frame, app, cols[1]);
}

fn draw_nodes(frame: &mut Frame, app: &App, area: Rect) {
    let nodes = app.node_list();
    let focused = app.focus == Focus::Nodes;
    let title = format!("Nodes ({})", nodes.len());
    let items: Vec<ListItem> = nodes
        .iter()
        .enumerate()
        .map(|(i, (id, label))| {
            let hit = app
                .graph
                .read()
                .unwrap()
                .node_matches_search(id);
            let mark = if focused && i == app.node_cursor {
                ">"
            } else {
                " "
            };
            let star = if hit { "*" } else { " " };
            let text = format!("{mark}{star}{}  {}", id, truncate(&label.to_string(), 24));
            let style = if focused && i == app.node_cursor {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if hit {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(text, style))
        })
        .collect();
    let border = if focused {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border)
            .title(title),
    );
    frame.render_widget(list, area);
}

fn draw_edges(frame: &mut Frame, app: &App, area: Rect) {
    let edges = app.incident_edges();
    let focused = app.focus == Focus::Edges;
    let sel = app
        .current_node_id()
        .map(|id| id.to_string())
        .unwrap_or_else(|| "-".into());
    let title = format!("Edges @ {sel}");
    let items: Vec<ListItem> = edges
        .iter()
        .enumerate()
        .map(|(i, (id, from, to))| {
            let mark = if focused && i == app.edge_cursor {
                ">"
            } else {
                " "
            };
            let text = format!("{mark}{}  {} -> {}", id, from, to);
            let style = if focused && i == app.edge_cursor {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(text, style))
        })
        .collect();
    let border = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border)
            .title(title),
    );
    frame.render_widget(list, area);
}

fn draw_graph(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Graph;
    let border = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border)
        .title("Graph");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(scene) = app.viz.scene.as_ref() {
        frame.render_widget(microdot_termgraph::SceneWidget { scene }, inner);
    } else {
        let msg = app
            .viz
            .last_error()
            .unwrap_or("empty graph — press i to insert a node");
        frame.render_widget(Paragraph::new(msg), inner);
    }
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    let g = app.graph.read().unwrap();
    let sel = g
        .current_node()
        .map(|id| {
            let label = g
                .find_node_label(id)
                .map(|l| l.to_string())
                .unwrap_or_default();
            format!("sel {} \"{}\"", id, truncate(&label, 20))
        })
        .unwrap_or_else(|| "sel -".into());
    let edge = app
        .selected_edge
        .as_ref()
        .map(|e| format!("edge {}", e))
        .unwrap_or_else(|| "edge -".into());
    let dirty = if app.dirty { "dirty" } else { "clean" };
    let status = format!(
        "{} · {} · {} · {} · {}",
        sel,
        edge,
        g.direction().to_string(),
        dirty,
        app.json_path().display()
    );
    drop(g);

    frame.render_widget(
        Paragraph::new(status).style(Style::default().fg(Color::Cyan)),
        chunks[0],
    );

    let help_line = if let Some(confirm) = app.confirm {
        match confirm {
            ConfirmKind::Quit => "quit with unsaved changes? [y/n]".to_string(),
            ConfirmKind::Delete { keep_edges: false } => {
                "delete node? [x/y] confirm  [n/Esc] cancel".to_string()
            }
            ConfirmKind::Delete { keep_edges: true } => {
                "delete keep edges? [X/y] confirm  [n/Esc] cancel".to_string()
            }
        }
    } else if app.link_pick {
        "link mode: j/k pick target · Enter link · Esc cancel".to_string()
    } else if let Some(draft) = &app.draft {
        format!("{}{}_", app.draft_prompt(), draft.buffer)
    } else if !app.note.is_empty() {
        app.note.clone()
    } else {
        "Tab focus · j/k move · / search · ` menu · : cmd · ? help · q quit".to_string()
    };

    frame.render_widget(
        Paragraph::new(help_line).wrap(Wrap { trim: true }),
        chunks[1],
    );
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let width = 60u16.min(area.width.saturating_sub(4));
    let height = 22u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let popup = Rect {
        x,
        y,
        width,
        height,
    };
    frame.render_widget(Clear, popup);
    let p = Paragraph::new(HELP_TEXT)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help — Esc to close"),
        );
    frame.render_widget(p, popup);
}

fn truncate(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{t}…")
    }
}
