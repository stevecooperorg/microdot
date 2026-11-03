//! Positioned graph scene — layout is external (e.g. Graphviz).

use crate::style::{EdgeStyle, NodeStyle};

/// How to shorten labels that would overflow the pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelMode {
    #[default]
    Full,
    Truncate(usize),
    /// Show the node id instead of the label text.
    Ids,
}

/// A node with absolute character-cell coordinates (top-left of the box).
#[derive(Debug, Clone)]
pub struct PlacedNode {
    pub id: String,
    pub label: String,
    pub x: usize,
    pub y: usize,
    pub style: NodeStyle,
}

/// An edge between placed nodes.
#[derive(Debug, Clone)]
pub struct PlacedEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub label: String,
    pub style: EdgeStyle,
    pub parallel_offset: i32,
}

/// Fully positioned scene ready for terminal rendering.
#[derive(Debug, Clone, Default)]
pub struct Scene {
    pub nodes: Vec<PlacedNode>,
    pub edges: Vec<PlacedEdge>,
    pub label_mode: LabelMode,
    pub padding: usize,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            padding: 1,
            ..Self::default()
        }
    }

    pub fn node(&self, id: &str) -> Option<&PlacedNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn node_mut(&mut self, id: &str) -> Option<&mut PlacedNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn edge_mut(&mut self, id: &str) -> Option<&mut PlacedEdge> {
        self.edges.iter_mut().find(|e| e.id == id)
    }

    pub fn reset_styles(&mut self) {
        for n in &mut self.nodes {
            n.style = NodeStyle::default();
        }
        for e in &mut self.edges {
            e.style = EdgeStyle::default();
        }
    }

    /// Assign parallel offsets for edges that share the same endpoint pair.
    pub fn assign_parallel_offsets(&mut self) {
        use std::collections::HashMap;
        let mut groups: HashMap<(String, String), Vec<usize>> = HashMap::new();
        for (i, e) in self.edges.iter().enumerate() {
            let key = if e.from <= e.to {
                (e.from.clone(), e.to.clone())
            } else {
                (e.to.clone(), e.from.clone())
            };
            groups.entry(key).or_default().push(i);
        }
        for idxs in groups.values() {
            if idxs.len() <= 1 {
                continue;
            }
            let count = idxs.len() as i32;
            for (j, &i) in idxs.iter().enumerate() {
                self.edges[i].parallel_offset = 2 * j as i32 - (count - 1);
            }
        }
    }
}
