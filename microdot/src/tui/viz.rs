//! TUI graph preview: Graphviz lays out, microdot-termgraph draws.

use super::theme::Theme;
use crate::graphviz::{self, DisplayMode, GraphVizExporter, PlainLayout};
use microdot_core::graph::Graph;
use microdot_core::Id;
use microdot_termgraph::{
    node_box_width, BoxBorder, LabelMode, PlacedEdge, PlacedNode, Scene, NODE_BOX_HEIGHT,
};
use std::collections::HashMap;

/// Characters per Graphviz inch when mapping to the terminal grid.
const COLS_PER_INCH: f64 = 12.0;
const ROWS_PER_INCH: f64 = 6.0;

pub struct VizState {
    pub scene: Option<Scene>,
    pub label_mode: LabelMode,
    theme: Theme,
    structure_epoch: u64,
    last_built_epoch: u64,
    last_error: Option<String>,
}

impl VizState {
    pub fn new(theme: Theme) -> Self {
        Self {
            scene: None,
            label_mode: LabelMode::Truncate(24),
            theme,
            structure_epoch: 0,
            last_built_epoch: u64::MAX,
            last_error: None,
        }
    }

    pub fn mark_structure_changed(&mut self) {
        self.structure_epoch = self.structure_epoch.wrapping_add(1);
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn rebuild_if_needed(&mut self, graph: &Graph) {
        if self.last_built_epoch == self.structure_epoch && self.scene.is_some() {
            return;
        }
        self.rebuild(graph);
    }

    pub fn rebuild(&mut self, graph: &Graph) {
        if graph.node_snapshots().is_empty() {
            self.scene = None;
            self.last_error = None;
            self.last_built_epoch = self.structure_epoch;
            return;
        }

        let mut exporter = GraphVizExporter::new(DisplayMode::Interactive);
        let dot = exporter.export_dot(graph);

        match graphviz::layout_dot_plain(&dot) {
            Ok(layout) => {
                self.scene = Some(scene_from_layout(
                    graph,
                    &layout,
                    self.label_mode,
                    self.theme,
                ));
                self.last_error = None;
            }
            Err(e) => {
                self.scene = None;
                self.last_error = Some(format!("graphviz layout failed: {e}"));
            }
        }
        self.last_built_epoch = self.structure_epoch;
    }

    pub fn apply_highlights(
        &mut self,
        selected_node: Option<&Id>,
        selected_edge: Option<&Id>,
        search_hits: &[Id],
    ) {
        let Some(scene) = self.scene.as_mut() else {
            return;
        };
        let theme = self.theme;
        let default_node = theme.default_node_style();
        let default_edge = theme.default_edge_style();
        for node in &mut scene.nodes {
            node.style = default_node.clone();
        }
        for edge in &mut scene.edges {
            edge.style = default_edge.clone();
        }

        let hit_ids: std::collections::HashSet<String> =
            search_hits.iter().map(|id| id.to_string()).collect();

        for node in &mut scene.nodes {
            if hit_ids.contains(&node.id) {
                node.style.border_color = theme.search_node_border;
                node.style.text_color = theme.search_node_text;
            }
            if selected_node.is_some_and(|id| id.to_string() == node.id) {
                node.style.border = BoxBorder::Rounded;
                node.style.border_color = theme.selected_node_border;
                node.style.text_color = theme.selected_node_text;
            }
        }

        if let Some(eid) = selected_edge {
            if let Some(edge) = scene.edge_mut(&eid.to_string()) {
                edge.style.line_color = theme.selected_edge;
                edge.style.text_color = theme.selected_edge;
            }
        }
    }

    pub fn set_truncate(&mut self) {
        self.label_mode = LabelMode::Truncate(24);
        self.mark_structure_changed();
    }

    pub fn set_numeric_ids(&mut self) {
        self.label_mode = LabelMode::Ids;
        self.mark_structure_changed();
    }
}

fn scene_from_layout(
    graph: &Graph,
    layout: &PlainLayout,
    label_mode: LabelMode,
    theme: Theme,
) -> Scene {
    let labels: HashMap<String, String> = graph
        .node_snapshots()
        .into_iter()
        .map(|(id, label)| (id.to_string(), label.to_string()))
        .collect();

    // Graphviz Y grows upward; terminal Y grows downward.
    let max_y = layout.height.max(
        layout
            .nodes
            .iter()
            .map(|n| n.y + n.height / 2.0)
            .fold(0.0, f64::max),
    );

    let mut scene = Scene::new();
    scene.label_mode = label_mode;
    scene.padding = 1;

    let node_style = theme.default_node_style();
    let edge_style = theme.default_edge_style();

    for n in &layout.nodes {
        let label = labels.get(&n.id).cloned().unwrap_or_else(|| n.id.clone());
        let display = match label_mode {
            LabelMode::Full => label.clone(),
            LabelMode::Truncate(max) => {
                let count = label.chars().count();
                if count <= max {
                    label.clone()
                } else {
                    format!(
                        "{}…",
                        label.chars().take(max.saturating_sub(1)).collect::<String>()
                    )
                }
            }
            LabelMode::Ids => n.id.clone(),
        };
        let box_w = node_box_width(&display);
        let cx = n.x * COLS_PER_INCH;
        let cy = (max_y - n.y) * ROWS_PER_INCH;
        let x = (cx - box_w as f64 / 2.0).round().max(0.0) as usize;
        let y = (cy - NODE_BOX_HEIGHT as f64 / 2.0).round().max(0.0) as usize;

        scene.nodes.push(PlacedNode {
            id: n.id.clone(),
            label,
            x,
            y,
            style: node_style.clone(),
        });
    }

    for (eid, from, to) in graph.edge_snapshots() {
        scene.edges.push(PlacedEdge {
            id: eid.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            label: String::new(),
            style: edge_style.clone(),
            parallel_offset: 0,
        });
    }
    scene.assign_parallel_offsets();
    scene
}

#[cfg(test)]
mod tests {
    use super::*;
    use microdot_core::Label;

    #[test]
    fn parse_plain_sample_builds_scene() {
        let plain = "\
graph 1 2.5 3.0
node n0 0.75 2.25 1.0 0.5 \"A\" solid black lightgrey
node n1 0.75 0.75 1.0 0.5 \"B\" solid black lightgrey
edge n0 n1 2 0.75 2.0 0.75 1.0
stop
";
        let layout = graphviz::parse_plain(plain).expect("parse");
        assert_eq!(layout.nodes.len(), 2);

        let mut graph = Graph::new();
        let (a, _) = graph.insert_node(Label::new("A"));
        let (b, _) = graph.insert_node(Label::new("B"));
        graph.link_edge(&a, &b);

        let scene = scene_from_layout(&graph, &layout, LabelMode::Full, Theme::night());
        assert_eq!(scene.nodes.len(), 2);
        assert_eq!(scene.edges.len(), 1);
    }
}
