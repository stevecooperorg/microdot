use crate::command::GraphCommand;
use crate::exporter::{Exporter, NodeHighlight};
use crate::{CommandResult, Id, Label};
use std::collections::BTreeSet;

#[derive(Default)]
pub struct Graph {
    node_high_water: usize,
    edge_high_water: usize,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    is_left_right: bool,
    current_search: Option<Label>,
    current_node: Option<Id>,
}

struct Node {
    id: Id,
    label: Label,
}

struct Edge {
    id: Id,
    from: Id,
    to: Id,
}

impl Graph {
    pub fn new() -> Self {
        Graph::default()
    }

    fn next_node_id(&mut self) -> Id {
        let id = format!("n{}", self.node_high_water);

        self.node_high_water += 1;

        Id::new(id)
    }

    fn next_edge_id(&mut self) -> Id {
        let id = format!("e{}", self.edge_high_water);

        self.edge_high_water += 1;

        Id::new(id)
    }

    fn find_edge_idx(&self, id: &Id) -> Option<usize> {
        self.edges
            .iter()
            .enumerate()
            .find(|(_, e)| &e.id == id)
            .map(|(idx, _)| idx)
    }

    fn find_node_idx(&self, id: &Id) -> Option<usize> {
        self.nodes
            .iter()
            .enumerate()
            .find(|(_, e)| &e.id == id)
            .map(|(idx, _)| idx)
    }

    pub fn export<X: Exporter>(&self, exporter: &mut X) {
        exporter.set_direction(self.is_left_right);

        for node in &self.nodes {
            let highlight = if self.node_matches_current_search(node) {
                NodeHighlight::SearchResult
            } else if self.current_node == Some(node.id.clone()) {
                NodeHighlight::CurrentNode
            } else {
                NodeHighlight::Normal
            };

            exporter.add_node(&node.id, &node.label, highlight);
        }

        for edge in &self.edges {
            exporter.add_edge(&edge.id, &edge.from, &edge.to);
        }
    }

    pub fn apply_command(&mut self, command: GraphCommand) -> CommandResult {
        match command {
            GraphCommand::DeleteNode { id, keep_edges } => self.delete_node(&id, keep_edges),
            GraphCommand::ExpandEdge { id, label } => self.expand_edge(&id, &label),
            GraphCommand::InsertAfterNode { id, label } => self.inject_after_node(&id, &label),
            GraphCommand::InsertBeforeNode { id, label } => self.inject_before_node(&id, &label),
            GraphCommand::InsertNode { label } => self.insert_node(label).1,
            GraphCommand::LinkEdge { from, to } => self.link_edge(&from, &to),
            GraphCommand::RenameNode { id, label } => self.rename_node(&id, label),
            GraphCommand::SelectNode { id } => self.select_node(&id),
            GraphCommand::SetDirection { is_left_right } => self.set_direction(is_left_right),
            GraphCommand::UnlinkEdge { id } => self.unlink_edge(&id),
        }
    }

    fn node_matches_current_search(&self, n: &Node) -> bool {
        match self.current_search.as_ref() {
            Some(current_search) => n.label.0.contains(&current_search.0),
            None => false,
        }
    }

    pub fn find_node_label(&self, id: &Id) -> Option<Label> {
        if let Some(idx) = self.find_node_idx(id) {
            if let Some(node) = self.nodes.get(idx) {
                return Some(node.label.clone());
            }
        }

        None
    }

    pub fn highlight_search_results(&mut self, sub_label: Label) -> CommandResult {
        self.current_search = Some(sub_label.clone());

        let mut matches: Vec<_> = self
            .nodes
            .iter()
            .filter(|n| self.node_matches_current_search(n))
            .collect();

        matches.sort_by_key(|n| &n.id.0);

        let search_lines: Vec<_> = matches
            .iter()
            .map(|n| format!("{}: {}", n.id, n.label))
            .collect();

        let msg = format!(
            "Search results for: {},\n{}\n",
            sub_label.0,
            search_lines.join("\n")
        );

        CommandResult::new(msg)
    }

    pub fn set_direction(&mut self, is_left_right: bool) -> CommandResult {
        self.is_left_right = is_left_right;
        CommandResult::new(format!(
            "Direction changed to {}",
            if is_left_right { "LR" } else { "TB" }
        ))
    }

    fn unlink_edge(&mut self, id: &Id) -> CommandResult {
        match self.find_edge_idx(id) {
            Some(idx) => {
                self.edges.remove(idx);

                CommandResult::new(format!("edge {} removed", id))
            }
            None => CommandResult::new(format!("edge {} not found", id)),
        }
    }

    fn rename_node(&mut self, id: &Id, label: Label) -> CommandResult {
        if let Some(idx) = self.find_node_idx(id) {
            self.current_node = Some(id.clone());

            if let Some(node) = self.nodes.get_mut(idx) {
                node.label = label.clone();

                CommandResult::new(format!("Node {} renamed to '{}'", id, label))
            } else {
                CommandResult::new(format!("Could not find node at index {}", idx))
            }
        } else {
            CommandResult::new(format!("Could not find node {}", id))
        }
    }

    pub fn insert_node(&mut self, label: Label) -> (Id, CommandResult) {
        let id = self.next_node_id();

        let node = Node {
            id: id.clone(),
            label: label.clone(),
        };

        self.nodes.push(node);
        self.current_node = Some(id.clone());

        (
            id.clone(),
            CommandResult::new(format!("inserted node {}: '{}'", id, label)),
        )
    }

    pub fn select_node(&mut self, id: &Id) -> CommandResult {
        if self.find_node_idx(id).is_none() {
            return CommandResult::new(format!("node {} not found", id));
        }

        self.current_node = Some(id.clone());
        CommandResult::new(format!("node {} selected", id))
    }

    pub fn inject_after_node(&mut self, from: &Id, label: &Label) -> CommandResult {
        if self.find_node_idx(from).is_none() {
            return CommandResult::new(format!("source node {} not found", from));
        }

        let (id, _) = self.insert_node(label.clone());

        self.link_edge(from, &id);
        self.current_node = Some(id.clone());

        CommandResult::new(format!("inserted node {}: '{}' after {}", id, label, from))
    }

    pub fn inject_before_node(&mut self, to: &Id, label: &Label) -> CommandResult {
        if self.find_node_idx(to).is_none() {
            return CommandResult::new(format!("target node {} not found", to));
        }

        let (id, _) = self.insert_node(label.clone());

        self.link_edge(&id, to);
        self.current_node = Some(id.clone());

        CommandResult::new(format!("inserted node {}: '{}' before {}", id, label, to))
    }

    pub fn expand_edge(&mut self, edge_id: &Id, label: &Label) -> CommandResult {
        let (from, to) = match self.find_edge_idx(edge_id) {
            Some(idx) => {
                let edge = &self.edges[idx];
                (edge.from.clone(), edge.to.clone())
            }
            None => return CommandResult::new(format!("edge {} not found", edge_id)),
        };

        self.unlink_edge(edge_id);
        let (new_id, _) = self.insert_node(label.clone());
        self.link_edge(&from, &new_id);
        self.link_edge(&new_id, &to);
        self.current_node = Some(new_id.clone());
        CommandResult::new(format!(
            "injected {}: '{}' between {} and {}",
            new_id, label, from, to
        ))
    }

    pub fn link_edge(&mut self, from: &Id, to: &Id) -> CommandResult {
        if self.find_node_idx(from).is_none() {
            return CommandResult::new(format!("source node {} not found", from));
        }

        if self.find_node_idx(to).is_none() {
            return CommandResult::new(format!("target node {} not found", to));
        }

        // we know both exist; create the edge
        let id = self.next_edge_id();

        let edge = Edge {
            id: id.clone(),
            from: from.clone(),
            to: to.clone(),
        };

        self.edges.push(edge);

        CommandResult::new(format!("Added edge {} from {} to {}", id, from, to))
    }

    fn delete_node(&mut self, id: &Id, keep_connected: bool) -> CommandResult {
        match self.find_node_idx(id) {
            Some(idx) => {
                // delete all edges to or from this node
                let mut edges_touching: BTreeSet<Id> = Default::default();

                let mut from_nodes: BTreeSet<Id> = Default::default();
                let mut to_nodes: BTreeSet<Id> = Default::default();

                for edge in &self.edges {
                    if &edge.from == id || &edge.to == id {
                        edges_touching.insert(edge.id.clone());
                        from_nodes.insert(edge.from.clone());
                        to_nodes.insert(edge.to.clone());
                    }
                }

                for delete in &edges_touching {
                    if let Some(idx) = self.find_edge_idx(delete) {
                        self.edges.remove(idx);
                    }
                }

                self.nodes.remove(idx);

                if self.current_node == Some(id.clone()) {
                    self.current_node = None;
                }

                if keep_connected {
                    for from in &from_nodes {
                        for to in &to_nodes {
                            self.link_edge(from, to);
                        }
                    }
                }

                CommandResult::new(format!("node {} removed", id))
            }
            None => CommandResult::new(format!("node {} not found", id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_insert_a_node() {
        let mut graph = Graph::new();
        let (id, _) = graph.insert_node(Label::new("a node label"));

        assert_eq!(graph.nodes.len(), 1);

        assert_eq!(graph.current_node, Some(id))
    }

    #[test]
    fn can_select_a_node() {
        let mut graph = Graph::new();
        let (id1, _) = graph.insert_node(Label::new("first node"));
        let (id2, _) = graph.insert_node(Label::new("second node"));

        assert_eq!(graph.current_node, Some(id2));
        graph.select_node(&id1);
        assert_eq!(graph.current_node, Some(id1));
    }

    #[test]
    fn can_delete_a_node() {
        let mut graph = Graph::new();
        let (id1, _) = graph.insert_node(Label::new("first node"));
        let (id2, _) = graph.insert_node(Label::new("second node"));
        let (id3, _) = graph.insert_node(Label::new("third node"));

        let _e12 = graph.link_edge(&id1, &id2);
        let _e23 = graph.link_edge(&id2, &id3);

        // delete the middle node - should delete the edges too
        graph.delete_node(&id2, false);

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn can_delete_a_node_but_keep_edges() {
        let mut graph = Graph::new();
        let (id1, _) = graph.insert_node(Label::new("first node"));
        let (id2, _) = graph.insert_node(Label::new("second node"));
        let (id3, _) = graph.insert_node(Label::new("third node"));

        let _e12 = graph.link_edge(&id1, &id2);
        let _e23 = graph.link_edge(&id2, &id3);

        // delete the middle node - should delete the edges too
        graph.delete_node(&id2, true);

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }
}
