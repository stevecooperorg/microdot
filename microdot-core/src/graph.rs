use crate::command::GraphCommand;
use crate::exporter::{Exporter, NodeHighlight};
use crate::pet::{GetWeight, PGraph};
use crate::util::generate_hash;
use crate::{CommandResult, Id, Label};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

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

pub struct Node {
    id: Id,
    label: Label,
}

impl Node {
    pub fn label(&self) -> &Label {
        &self.label
    }
}

#[derive(Debug, Clone)]
pub enum VariableValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Time(Time),
}

impl Eq for VariableValue {}

impl PartialEq for VariableValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VariableValue::String(s1), VariableValue::String(s2)) => s1 == s2,
            (VariableValue::Number(n1), VariableValue::Number(n2)) => n1 == n2,
            (VariableValue::Boolean(b1), VariableValue::Boolean(b2)) => b1 == b2,
            (VariableValue::Time(t1), VariableValue::Time(t2)) => t1 == t2,
            _ => false,
        }
    }
}
impl Hash for VariableValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            VariableValue::String(s) => s.hash(state),
            VariableValue::Number(n) => n.to_string().hash(state),
            VariableValue::Boolean(b) => b.hash(state),
            VariableValue::Time(t) => t.to_string().hash(state),
        }
    }
}

impl VariableValue {
    pub fn as_string(&self) -> String {
        match self {
            VariableValue::String(s) => s.clone(),
            VariableValue::Number(n) => n.to_string(),
            VariableValue::Boolean(b) => b.to_string(),
            VariableValue::Time(t) => t.to_string(),
        }
    }

    pub fn infer(value: impl Into<String>) -> Self {
        let value = value.into();
        if value == "true" || value == "false" {
            VariableValue::Boolean(value.parse().unwrap())
        } else if let Ok(n) = value.parse() {
            VariableValue::Number(n)
        } else if let Some(time) = Time::parse(&value) {
            VariableValue::Time(time)
        } else {
            VariableValue::String(value)
        }
    }

    pub fn boolean(value: bool) -> Self {
        VariableValue::Boolean(value)
    }

    pub fn number(value: f64) -> Self {
        VariableValue::Number(value)
    }

    pub fn string(value: impl Into<String>) -> Self {
        VariableValue::String(value.into())
    }

    pub fn time(value: Time) -> Self {
        VariableValue::Time(value)
    }
}

impl Display for VariableValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            VariableValue::String(s) => s.to_string(),
            VariableValue::Number(n) => n.to_string(),
            VariableValue::Boolean(b) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            VariableValue::Time(t) => format!("{}", t),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]

pub enum Time {
    Minute(u32),
    Hour(u32),
    Day(u32),
    Month(u32),
    Year(u32),
}

impl PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        self.to_minutes() == other.to_minutes()
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Time::Minute(m) => write!(f, "{} minutes", m),
            Time::Hour(h) => write!(f, "{} hours", h),
            Time::Day(d) => write!(f, "{} days", d),
            Time::Month(m) => write!(f, "{} months", m),
            Time::Year(y) => write!(f, "{} years", y),
        }
    }
}

impl Time {
    pub fn to_minutes(&self) -> u32 {
        match self {
            Time::Minute(m) => *m,
            Time::Hour(h) => h * 60,
            Time::Day(d) => d * 60 * 24,
            Time::Month(m) => m * 60 * 24 * 30,
            Time::Year(y) => y * 60 * 24 * 365,
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        let rx = Regex::new(r"(\d+)\s*(m|h|d|M|y)").expect("not a regex");
        if let Some(caps) = rx.captures(input) {
            let value = caps.get(1).unwrap().as_str().parse().unwrap();
            let unit = caps.get(2).unwrap().as_str();
            match unit {
                "m" => Some(Time::Minute(value)),
                "h" => Some(Time::Hour(value)),
                "d" => Some(Time::Day(value)),
                "M" => Some(Time::Month(value)),
                "y" => Some(Time::Year(value)),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Variable {
    pub name: String,
    pub value: VariableValue,
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.name, self.value)
    }
}

impl Variable {
    pub fn new(name: impl Into<String>, value: VariableValue) -> Self {
        Variable {
            name: name.into(),
            value,
        }
    }
    pub fn boolean(name: impl Into<String>, value: bool) -> Self {
        Variable {
            name: name.into(),
            value: VariableValue::Boolean(value),
        }
    }

    pub fn variable_rx() -> Regex {
        Regex::new("\\$([A-Za-z][A-Za-z0-9_-]*)=([A-Za-z0-9_-]+)").expect("not a regex")
    }

    pub fn parse(input: &str) -> Option<Self> {
        let rx = Variable::variable_rx();
        if let Some(caps) = rx.captures(input) {
            let name = caps.get(1).unwrap().as_str();
            let value = caps.get(2).unwrap().as_str();
            let value = VariableValue::infer(value);
            Some(Variable::new(name, value))
        } else {
            None
        }
    }

    pub fn hash(&self) -> usize {
        generate_hash(&format!("{}", self))
    }
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

    pub fn to_petgraph(&self, graph: &mut PGraph, get_weights: impl GetWeight<Node>) {
        let mut indexes = BTreeMap::new();
        for node in &self.nodes {
            let weight = get_weights.get_weight(node);
            let id = graph.add_node(node.id.clone(), weight);

            indexes.insert(node.id.clone(), id);
        }

        for edge in &self.edges {
            let from = *indexes.get(&edge.from).unwrap();
            let to = *indexes.get(&edge.to).unwrap();

            graph.add_edge(from, to);
        }
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

    pub fn sources(&self) -> Vec<Id> {
        // find nodes with no incoming edges
        let mut node_ids = self.nodes.iter().map(|n| &n.id).collect::<BTreeSet<_>>();
        let port_ids = self.edges.iter().map(|e| &e.to).collect::<BTreeSet<_>>();
        // find the difference
        for edge in &port_ids {
            node_ids.remove(edge);
        }
        node_ids.into_iter().cloned().collect()
    }

    pub fn sinks(&self) -> Vec<Id> {
        // find nodes with no incoming edges
        let mut node_ids = self.nodes.iter().map(|n| &n.id).collect::<BTreeSet<_>>();
        let port_ids = self.edges.iter().map(|e| &e.from).collect::<BTreeSet<_>>();
        // find the difference
        for edge in &port_ids {
            node_ids.remove(edge);
        }
        node_ids.into_iter().cloned().collect()
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
