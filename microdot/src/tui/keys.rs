//! Key handling for the microdot TUI.

use super::app::{App, ConfirmKind, DraftKind, Focus};
use super::menu::MenuGroup;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle a key. Returns true when the TUI should quit.
pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
        return true;
    }

    // Confirm modal
    if let Some(confirm) = app.confirm {
        return match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => app.confirm_yes(),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.confirm = None;
                app.note("cancelled");
                false
            }
            KeyCode::Char('x') if matches!(confirm, ConfirmKind::Delete { keep_edges: false }) => {
                app.confirm_yes()
            }
            KeyCode::Char('X') if matches!(confirm, ConfirmKind::Delete { keep_edges: true }) => {
                app.confirm_yes()
            }
            _ => false,
        };
    }

    // Help overlay
    if app.show_help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                app.show_help = false;
            }
            _ => {}
        }
        return false;
    }

    // Menu
    if app.menu_open {
        return handle_menu(app, key);
    }

    // Draft editor
    if app.draft.is_some() {
        return handle_draft(app, key);
    }

    // Link pick mode
    if app.link_pick {
        return handle_link_pick(app, key);
    }

    handle_normal(app, key)
}

fn handle_menu(app: &mut App, key: KeyEvent) -> bool {
    let groups = MenuGroup::ALL.len();
    match key.code {
        KeyCode::Esc | KeyCode::Char('`') => {
            app.menu_open = false;
            false
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if app.menu_group > 0 {
                app.menu_group -= 1;
            } else {
                app.menu_group = groups - 1;
            }
            app.menu_item = 0;
            false
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.menu_group = (app.menu_group + 1) % groups;
            app.menu_item = 0;
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let n = app.active_menu_items().len();
            if n > 0 {
                if app.menu_item == 0 {
                    app.menu_item = n - 1;
                } else {
                    app.menu_item -= 1;
                }
            }
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let n = app.active_menu_items().len();
            if n > 0 {
                app.menu_item = (app.menu_item + 1) % n;
            }
            false
        }
        KeyCode::Enter => {
            let items = app.active_menu_items();
            if items.is_empty() {
                return false;
            }
            let action = items[app.menu_item.min(items.len() - 1)];
            app.menu_open = false;
            app.run_menu_action(action)
        }
        _ => false,
    }
}

fn handle_draft(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.draft = None;
            app.note("cancelled");
            false
        }
        KeyCode::Enter => app.commit_draft(),
        KeyCode::Backspace => {
            if let Some(d) = app.draft.as_mut() {
                d.buffer.pop();
            }
            false
        }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(d) = app.draft.as_mut() {
                d.buffer.push(c);
            }
            false
        }
        _ => false,
    }
}

fn handle_link_pick(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.link_pick = false;
            app.link_from = None;
            app.note("link cancelled");
            false
        }
        KeyCode::Enter => app.link_to_cursor(),
        KeyCode::Up | KeyCode::Char('k') => {
            app.move_node_cursor(-1);
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.move_node_cursor(1);
            false
        }
        KeyCode::Char('K') => {
            app.move_node_cursor(-10);
            false
        }
        KeyCode::Char('J') => {
            app.move_node_cursor(10);
            false
        }
        _ => false,
    }
}

fn handle_normal(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') => {
            if app.dirty {
                app.confirm = Some(ConfirmKind::Quit);
                app.note("quit? unsaved changes — y/n");
                false
            } else {
                true
            }
        }
        KeyCode::Char('`') => {
            app.menu_open = true;
            app.menu_item = 0;
            false
        }
        KeyCode::Char('?') => {
            app.show_help = true;
            false
        }
        KeyCode::Char(':') => {
            app.open_draft(DraftKind::Command);
            false
        }
        KeyCode::Char('/') => {
            app.open_draft(DraftKind::Search);
            false
        }
        KeyCode::Char('.') => {
            app.jump_search(true);
            false
        }
        KeyCode::Tab => {
            app.focus = app.focus.next();
            false
        }
        KeyCode::Left => {
            app.focus = Focus::Nodes;
            false
        }
        KeyCode::Right => {
            app.focus = Focus::Graph;
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            move_focus(app, -1);
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            move_focus(app, 1);
            false
        }
        KeyCode::Char('K') => {
            move_focus(app, -10);
            false
        }
        KeyCode::Char('J') => {
            move_focus(app, 10);
            false
        }
        KeyCode::Char('i') => {
            app.open_draft(DraftKind::Insert);
            false
        }
        KeyCode::Char('a') => {
            app.open_draft(DraftKind::After);
            false
        }
        KeyCode::Char('b') => {
            app.open_draft(DraftKind::Before);
            false
        }
        KeyCode::Char('r') => {
            app.open_draft(DraftKind::Rename);
            false
        }
        KeyCode::Char('x') => {
            app.confirm = Some(ConfirmKind::Delete { keep_edges: false });
            app.note("delete node? press x or y to confirm");
            false
        }
        KeyCode::Char('X') => {
            app.confirm = Some(ConfirmKind::Delete { keep_edges: true });
            app.note("delete keep edges? press X or y to confirm");
            false
        }
        KeyCode::Char('l') => {
            app.begin_link_pick();
            false
        }
        KeyCode::Char('u') => app.unlink_current(),
        KeyCode::Char('e') => {
            app.open_draft(DraftKind::Expand);
            false
        }
        _ => false,
    }
}

fn move_focus(app: &mut App, delta: isize) {
    match app.focus {
        Focus::Nodes => app.move_node_cursor(delta),
        Focus::Edges => app.move_edge_cursor(delta),
        Focus::Graph => {
            let _ = delta;
            // Graphviz owns layout; Relayout is available from the View menu.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use microdot_core::graph::Graph;
    use std::sync::{Arc, RwLock};
    use tokio::sync::mpsc;

    fn test_app() -> App {
        let (tx, _rx) = mpsc::unbounded_channel();
        App::new(
            Arc::new(RwLock::new(Graph::new())),
            std::path::PathBuf::from("/tmp/microdot-tui-test.json"),
            tx,
            crate::tui::Theme::night(),
        )
    }

    fn key(c: char) -> KeyEvent {
        KeyEvent::from(KeyCode::Char(c))
    }

    #[test]
    fn insert_via_draft() {
        let mut app = test_app();
        assert!(!handle_key(&mut app, key('i')));
        assert!(app.draft.is_some());
        for c in ['H', 'i'] {
            assert!(!handle_key(&mut app, key(c)));
        }
        assert!(!handle_key(
            &mut app,
            KeyEvent::from(KeyCode::Enter)
        ));
        let nodes = app.node_list();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].1.to_string(), "Hi");
    }

    #[test]
    fn tab_cycles_focus() {
        let mut app = test_app();
        assert_eq!(app.focus, Focus::Nodes);
        handle_key(&mut app, KeyEvent::from(KeyCode::Tab));
        assert_eq!(app.focus, Focus::Edges);
        handle_key(&mut app, KeyEvent::from(KeyCode::Tab));
        assert_eq!(app.focus, Focus::Graph);
        handle_key(&mut app, KeyEvent::from(KeyCode::Tab));
        assert_eq!(app.focus, Focus::Nodes);
    }

    #[test]
    fn edge_panel_arrows_select_edges_without_changing_node() {
        let mut graph = Graph::new();
        let (node, _) = graph.insert_node(microdot_core::Label::new("node"));
        let (first_target, _) = graph.insert_node(microdot_core::Label::new("first"));
        let (second_target, _) = graph.insert_node(microdot_core::Label::new("second"));
        graph.link_edge(&node, &first_target);
        graph.link_edge(&node, &second_target);
        graph.select_node(&node);

        let (tx, _rx) = mpsc::unbounded_channel();
        let graph = Arc::new(RwLock::new(graph));
        let mut app = App::new(
            graph,
            std::path::PathBuf::from("/tmp/microdot-tui-edge-test.json"),
            tx,
            crate::tui::Theme::night(),
        );
        let edges = app.incident_edges();
        assert_eq!(edges.len(), 2);
        assert_eq!(app.current_node_id(), Some(node.clone()));
        assert_eq!(app.selected_edge, Some(edges[0].0.clone()));

        app.focus = Focus::Edges;
        assert!(!handle_key(&mut app, KeyEvent::from(KeyCode::Down)));

        assert_eq!(app.current_node_id(), Some(node));
        assert_eq!(app.edge_cursor, 1);
        assert_eq!(app.selected_edge, Some(edges[1].0.clone()));
    }
}
