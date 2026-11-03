//! Application state for the microdot TUI.

use super::menu::{MenuAction, MenuGroup};
use super::viz::VizState;
use crate::dispatch::{dispatch_command, DispatchResult};
use crate::parser::parse_line;
use crate::persist::persist_locked;
use crate::Command;
use microdot_core::command::GraphCommand;
use microdot_core::graph::Graph;
use microdot_core::{Direction, Id, Label, Line};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Focus {
    Nodes,
    Edges,
    Graph,
}

impl Focus {
    pub fn next(self) -> Self {
        match self {
            Focus::Nodes => Focus::Edges,
            Focus::Edges => Focus::Graph,
            Focus::Graph => Focus::Nodes,
        }
    }
}

#[derive(Clone, Debug)]
pub enum DraftKind {
    Search,
    Insert,
    After,
    Before,
    Rename,
    Expand,
    CriticalPath,
    Cost,
    Command,
}

#[derive(Clone, Debug)]
pub struct Draft {
    pub kind: DraftKind,
    pub buffer: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfirmKind {
    Quit,
    Delete { keep_edges: bool },
}

pub struct App {
    pub graph: Arc<RwLock<Graph>>,
    pub json_file: PathBuf,
    pub reload_tx: UnboundedSender<()>,
    pub focus: Focus,
    pub selected_edge: Option<Id>,
    pub node_cursor: usize,
    pub edge_cursor: usize,
    pub dirty: bool,
    pub note: String,
    pub log_lines: Vec<String>,
    pub draft: Option<Draft>,
    pub confirm: Option<ConfirmKind>,
    pub link_pick: bool,
    pub link_from: Option<Id>,
    pub menu_open: bool,
    pub menu_group: usize,
    pub menu_item: usize,
    pub show_help: bool,
    pub last_search: Option<String>,
    pub viz: VizState,
}

impl App {
    pub fn new(
        graph: Arc<RwLock<Graph>>,
        json_file: PathBuf,
        reload_tx: UnboundedSender<()>,
    ) -> Self {
        let mut app = Self {
            graph,
            json_file,
            reload_tx,
            focus: Focus::Nodes,
            selected_edge: None,
            node_cursor: 0,
            edge_cursor: 0,
            dirty: false,
            note: String::new(),
            log_lines: Vec::new(),
            draft: None,
            confirm: None,
            link_pick: false,
            link_from: None,
            menu_open: false,
            menu_group: 0,
            menu_item: 0,
            show_help: false,
            last_search: None,
            viz: VizState::default(),
        };
        app.sync_selection_from_graph();
        app.rebuild_viz();
        app
    }

    pub fn json_path(&self) -> &Path {
        &self.json_file
    }

    pub fn push_log(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        self.note = msg.clone();
        self.log_lines.push(msg);
        if self.log_lines.len() > 200 {
            self.log_lines.drain(0..self.log_lines.len() - 200);
        }
    }

    pub fn note(&mut self, msg: impl Into<String>) {
        self.note = msg.into();
    }

    pub fn node_list(&self) -> Vec<(Id, Label)> {
        self.graph.read().unwrap().node_snapshots()
    }

    pub fn incident_edges(&self) -> Vec<(Id, Id, Id)> {
        let g = self.graph.read().unwrap();
        match g.current_node() {
            Some(id) => g.edges_incident_to(id),
            None => Vec::new(),
        }
    }

    pub fn current_node_id(&self) -> Option<Id> {
        self.graph.read().unwrap().current_node().cloned()
    }

    pub fn sync_selection_from_graph(&mut self) {
        let nodes = self.node_list();
        if nodes.is_empty() {
            self.node_cursor = 0;
            self.selected_edge = None;
            self.edge_cursor = 0;
            return;
        }
        let current = self.current_node_id();
        if let Some(cur) = current {
            if let Some(idx) = nodes.iter().position(|(id, _)| id == &cur) {
                self.node_cursor = idx;
            }
        } else {
            self.node_cursor = self.node_cursor.min(nodes.len() - 1);
            let id = nodes[self.node_cursor].0.clone();
            self.select_node_quiet(&id);
        }
        self.refresh_edge_selection();
    }

    pub fn refresh_edge_selection(&mut self) {
        let edges = self.incident_edges();
        if edges.is_empty() {
            self.selected_edge = None;
            self.edge_cursor = 0;
            return;
        }
        if let Some(sel) = &self.selected_edge {
            if let Some(idx) = edges.iter().position(|(id, _, _)| id == sel) {
                self.edge_cursor = idx;
                return;
            }
        }
        self.edge_cursor = 0;
        self.selected_edge = Some(edges[0].0.clone());
    }

    fn select_node_quiet(&mut self, id: &Id) {
        let mut g = self.graph.write().unwrap();
        let _ = g.select_node(id);
    }

    pub fn move_node_cursor(&mut self, delta: isize) {
        let nodes = self.node_list();
        if nodes.is_empty() {
            return;
        }
        let len = nodes.len() as isize;
        let mut idx = self.node_cursor as isize + delta;
        if idx < 0 {
            idx = 0;
        } else if idx >= len {
            idx = len - 1;
        }
        self.node_cursor = idx as usize;
        let id = nodes[self.node_cursor].0.clone();
        self.select_node_quiet(&id);
        self.refresh_edge_selection();
        self.refresh_viz_highlights();
    }

    pub fn move_edge_cursor(&mut self, delta: isize) {
        let edges = self.incident_edges();
        if edges.is_empty() {
            return;
        }
        let len = edges.len() as isize;
        let mut idx = self.edge_cursor as isize + delta;
        if idx < 0 {
            idx = 0;
        } else if idx >= len {
            idx = len - 1;
        }
        self.edge_cursor = idx as usize;
        let (eid, from, _) = edges[self.edge_cursor].clone();
        self.select_node_quiet(&from);
        self.selected_edge = Some(eid.clone());
        let edges = self.incident_edges();
        if let Some(i) = edges.iter().position(|(id, _, _)| id == &eid) {
            self.edge_cursor = i;
        }
        self.refresh_viz_highlights();
    }

    pub fn apply_command(&mut self, command: Command) -> bool {
        let result = dispatch_command(command, &self.graph, &self.json_file);
        self.take_dispatch(result)
    }

    pub fn take_dispatch(&mut self, result: DispatchResult) -> bool {
        for m in result.messages {
            self.push_log(m);
        }
        if result.dirty {
            self.dirty = true;
            self.viz.mark_structure_changed();
            self.sync_selection_from_graph();
            self.rebuild_viz();
        }
        result.quit
    }

    pub fn rebuild_viz(&mut self) {
        {
            let g = self.graph.read().unwrap();
            self.viz.rebuild_if_needed(&g);
        }
        self.refresh_viz_highlights();
    }

    pub fn refresh_viz_highlights(&mut self) {
        let g = self.graph.read().unwrap();
        let selected = g.current_node().cloned();
        let hits: Vec<Id> = g
            .node_snapshots()
            .into_iter()
            .filter(|(id, _)| g.node_matches_search(id))
            .map(|(id, _)| id)
            .collect();
        drop(g);
        self.viz
            .apply_highlights(selected.as_ref(), self.selected_edge.as_ref(), &hits);
    }

    pub fn persist(&mut self, compile_svg: bool) {
        match persist_locked(
            &self.json_file,
            &self.graph,
            compile_svg,
            Some(&self.reload_tx),
        ) {
            Ok(msg) => {
                self.dirty = false;
                self.push_log(msg);
            }
            Err(e) => self.note(format!("persist failed: {e}")),
        }
    }

    pub fn open_draft(&mut self, kind: DraftKind) {
        self.draft = Some(Draft {
            kind,
            buffer: String::new(),
        });
    }

    pub fn commit_draft(&mut self) -> bool {
        let Some(draft) = self.draft.take() else {
            return false;
        };
        let text = draft.buffer.trim().to_string();
        match draft.kind {
            DraftKind::Search => {
                if text.is_empty() {
                    return false;
                }
                self.last_search = Some(text.clone());
                let quit = self.apply_command(Command::Search {
                    sub_label: Label::new(text),
                });
                self.jump_search(false);
                quit
            }
            DraftKind::Insert => {
                if text.is_empty() {
                    return false;
                }
                self.apply_command(Command::GraphCommand(GraphCommand::InsertNode {
                    label: Label::new(text),
                }))
            }
            DraftKind::After => {
                let Some(id) = self.current_node_id() else {
                    self.note("no node selected");
                    return false;
                };
                if text.is_empty() {
                    return false;
                }
                self.apply_command(Command::GraphCommand(GraphCommand::InsertAfterNode {
                    id,
                    label: Label::new(text),
                }))
            }
            DraftKind::Before => {
                let Some(id) = self.current_node_id() else {
                    self.note("no node selected");
                    return false;
                };
                if text.is_empty() {
                    return false;
                }
                self.apply_command(Command::GraphCommand(GraphCommand::InsertBeforeNode {
                    id,
                    label: Label::new(text),
                }))
            }
            DraftKind::Rename => {
                let Some(id) = self.current_node_id() else {
                    self.note("no node selected");
                    return false;
                };
                if text.is_empty() {
                    return false;
                }
                self.apply_command(Command::GraphCommand(GraphCommand::RenameNode {
                    id,
                    label: Label::new(text),
                }))
            }
            DraftKind::Expand => {
                let Some(eid) = self.selected_edge.clone() else {
                    self.note("no edge selected");
                    return false;
                };
                if text.is_empty() {
                    return false;
                }
                self.apply_command(Command::GraphCommand(GraphCommand::ExpandEdge {
                    id: eid,
                    label: Label::new(text),
                }))
            }
            DraftKind::CriticalPath => {
                if text.is_empty() {
                    return false;
                }
                self.apply_command(Command::CriticalPathAnalysis {
                    variable_name: text,
                })
            }
            DraftKind::Cost => {
                if text.is_empty() {
                    return false;
                }
                self.apply_command(Command::CostAnalysis {
                    variable_name: text,
                })
            }
            DraftKind::Command => {
                if text.is_empty() {
                    return false;
                }
                let command = parse_line(Line::new(text));
                self.apply_command(command)
            }
        }
    }

    pub fn jump_search(&mut self, after_current: bool) {
        let Some(query) = self.last_search.clone() else {
            self.note("no previous search");
            return;
        };
        let nodes = self.node_list();
        if nodes.is_empty() {
            return;
        }
        let q = query.to_lowercase();
        let start = if after_current {
            (self.node_cursor + 1) % nodes.len()
        } else {
            self.node_cursor
        };
        let mut found = None;
        for offset in 0..nodes.len() {
            let idx = (start + offset) % nodes.len();
            if nodes[idx].1.to_string().to_lowercase().contains(&q) {
                found = Some(idx);
                break;
            }
        }
        if let Some(idx) = found {
            self.node_cursor = idx;
            let id = nodes[idx].0.clone();
            self.select_node_quiet(&id);
            self.refresh_edge_selection();
            self.refresh_viz_highlights();
            self.note(format!("match: {}", nodes[idx].1));
        } else {
            self.note(format!("no match for '{query}'"));
        }
    }

    pub fn begin_link_pick(&mut self) {
        match self.current_node_id() {
            Some(id) => {
                self.link_from = Some(id);
                self.link_pick = true;
                self.focus = Focus::Nodes;
                self.note("link: pick target node, Enter to link, Esc cancel");
            }
            None => self.note("no node selected"),
        }
    }

    pub fn link_to_cursor(&mut self) -> bool {
        let Some(from) = self.link_from.take() else {
            self.link_pick = false;
            self.note("link cancelled");
            return false;
        };
        self.link_pick = false;
        let nodes = self.node_list();
        if nodes.is_empty() {
            return false;
        }
        let to = nodes[self.node_cursor].0.clone();
        if from == to {
            self.note("cannot link node to itself");
            return false;
        }
        self.apply_command(Command::GraphCommand(GraphCommand::LinkEdge { from, to }))
    }

    pub fn unlink_current(&mut self) -> bool {
        let Some(eid) = self.selected_edge.clone() else {
            self.note("no edge selected");
            return false;
        };
        self.apply_command(Command::GraphCommand(GraphCommand::UnlinkEdge { id: eid }))
    }

    pub fn delete_current(&mut self, keep_edges: bool) -> bool {
        let Some(id) = self.current_node_id() else {
            self.note("no node selected");
            return false;
        };
        self.apply_command(Command::GraphCommand(GraphCommand::DeleteNode {
            id,
            keep_edges,
        }))
    }

    pub fn confirm_yes(&mut self) -> bool {
        match self.confirm.take() {
            Some(ConfirmKind::Quit) => {
                if self.dirty {
                    self.persist(true);
                }
                true
            }
            Some(ConfirmKind::Delete { keep_edges }) => {
                let _ = self.delete_current(keep_edges);
                false
            }
            None => false,
        }
    }

    pub fn set_direction(&mut self, direction: Direction) -> bool {
        self.apply_command(Command::GraphCommand(GraphCommand::SetDirection {
            direction,
        }))
    }

    pub fn run_menu_action(&mut self, action: MenuAction) -> bool {
        match action {
            MenuAction::Save => {
                self.persist(true);
                false
            }
            MenuAction::ShowSvg => self.apply_command(Command::Show),
            MenuAction::PrintDot => self.apply_command(Command::PrintDot),
            MenuAction::PrintJson => self.apply_command(Command::PrintJson),
            MenuAction::Quit => {
                if self.dirty {
                    self.confirm = Some(ConfirmKind::Quit);
                    false
                } else {
                    true
                }
            }
            MenuAction::Insert => {
                self.open_draft(DraftKind::Insert);
                false
            }
            MenuAction::After => {
                self.open_draft(DraftKind::After);
                false
            }
            MenuAction::Before => {
                self.open_draft(DraftKind::Before);
                false
            }
            MenuAction::Rename => {
                self.open_draft(DraftKind::Rename);
                false
            }
            MenuAction::Delete => {
                self.confirm = Some(ConfirmKind::Delete { keep_edges: false });
                false
            }
            MenuAction::DeleteKeepEdges => {
                self.confirm = Some(ConfirmKind::Delete { keep_edges: true });
                false
            }
            MenuAction::Link => {
                self.begin_link_pick();
                false
            }
            MenuAction::Unlink => self.unlink_current(),
            MenuAction::Expand => {
                self.open_draft(DraftKind::Expand);
                false
            }
            MenuAction::DirLr => self.set_direction(Direction::LR),
            MenuAction::DirRl => self.set_direction(Direction::RL),
            MenuAction::DirTb => self.set_direction(Direction::TB),
            MenuAction::DirBt => self.set_direction(Direction::BT),
            MenuAction::CriticalPath => {
                self.open_draft(DraftKind::CriticalPath);
                false
            }
            MenuAction::Cost => {
                self.open_draft(DraftKind::Cost);
                false
            }
            MenuAction::Relayout => {
                self.viz.mark_structure_changed();
                self.rebuild_viz();
                self.note("relayout done");
                false
            }
            MenuAction::TruncateLabels => {
                self.viz.set_truncate();
                self.rebuild_viz();
                self.note("scaling: truncate labels");
                false
            }
            MenuAction::NumericIds => {
                self.viz.set_numeric_ids();
                self.rebuild_viz();
                self.note("scaling: numeric ids");
                false
            }
            MenuAction::KeyHelp => {
                self.show_help = true;
                false
            }
        }
    }

    pub fn active_menu_items(&self) -> &'static [MenuAction] {
        MenuGroup::ALL[self.menu_group].items()
    }

    pub fn draft_prompt(&self) -> &'static str {
        match self.draft.as_ref().map(|d| &d.kind) {
            Some(DraftKind::Search) => "/",
            Some(DraftKind::Insert) => "insert> ",
            Some(DraftKind::After) => "after> ",
            Some(DraftKind::Before) => "before> ",
            Some(DraftKind::Rename) => "rename> ",
            Some(DraftKind::Expand) => "expand> ",
            Some(DraftKind::CriticalPath) => "crit> ",
            Some(DraftKind::Cost) => "cost> ",
            Some(DraftKind::Command) => ":",
            None => "",
        }
    }
}
