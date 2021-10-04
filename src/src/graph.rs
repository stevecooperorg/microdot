use crate::{CommandResult, Exporter, GraphCommand, Id, Label};

pub struct Graph {
    node_high_water: usize,
    edge_high_water: usize,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    is_left_right: bool,
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

impl Default for Graph {
    fn default() -> Self {
        Self {
            node_high_water: 0,
            edge_high_water: 0,
            nodes: vec![],
            edges: vec![],
            is_left_right: false,
        }
    }
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
            exporter.add_node(&node.id, &node.label);
        }

        for edge in &self.edges {
            exporter.add_edge(&edge.id, &edge.from, &edge.to);
        }
    }

    pub fn apply_command(&mut self, command: GraphCommand) -> CommandResult {
        match command {
            GraphCommand::InsertNode { label } => self.insert_node(label).1,
            GraphCommand::DeleteNode { id } => self.delete_node(&id),
            GraphCommand::LinkEdge { from, to } => self.link_edge(&from, &to),
            GraphCommand::RenameNode { id, label } => self.rename_node(&id, label),
            GraphCommand::UnlinkEdge { id } => self.unlink_edge(&id),
            GraphCommand::SetDirection { is_left_right } => self.set_direction(is_left_right),
            GraphCommand::InsertAfterNode { id, label } => self.inject_after_node(&id, &label),
            GraphCommand::InsertBeforeNode {id, label } => self.inject_before_node(&id,&label),
            GraphCommand::ExpandEdge { id, label} => self.expand_edge(&id,&label)
        }
    }

    pub fn set_direction(&mut self, is_left_right: bool) -> CommandResult {
        self.is_left_right = is_left_right;
        CommandResult::new(format!(
            "Direction changed to {}",
            if is_left_right { "LR" } else { "TB" }
        ))
    }

    fn unlink_edge(&mut self, id: &Id) -> CommandResult {
        match self.find_edge_idx(&id) {
            Some(idx) => {
                self.edges.remove(idx);

                CommandResult::new(format!("edge {} removed", id))
            }
            None => CommandResult::new(format!("edge {} not found", id)),
        }
    }

    fn rename_node(&mut self, id: &Id, label: Label) -> CommandResult {
        if let Some(idx) = self.find_node_idx(&id) {
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

        (
            id.clone(),
            CommandResult::new(format!("inserted node {}: '{}'", id, label)),
        )
    }

    pub fn inject_after_node(&mut self, from: &Id, label: &Label) -> CommandResult {
        if !self.find_node_idx(&from).is_some() {
            return CommandResult::new(format!("source node {} not found", from));
        }

        let (id, _) = self.insert_node(label.clone());

        self.link_edge(from, &id);

        CommandResult::new(format!("inserted node {}: '{}' after {}", id, label, from))
    }

    pub fn inject_before_node(&mut self, to: &Id, label: &Label) -> CommandResult {
        if !self.find_node_idx(&to).is_some() {
            return CommandResult::new(format!("target node {} not found", to));
        }

        let (id, _) = self.insert_node(label.clone());

        self.link_edge(&id, to);

        CommandResult::new(format!("inserted node {}: '{}' before {}", id, label, to))
    }

    pub fn expand_edge(&mut self, edge_id: &Id, label: &Label) -> CommandResult {
        let (from, to) =  match self.find_edge_idx(&edge_id) {
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
        CommandResult::new(format!("injected {}: '{}' between {} and {}", new_id, label, from, to))
    }

    pub fn link_edge(&mut self, from: &Id, to: &Id) -> CommandResult {
        if !self.find_node_idx(&from).is_some() {
            return CommandResult::new(format!("source node {} not found", from));
        }

        if !self.find_node_idx(&to).is_some() {
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

    fn delete_node(&mut self, id: &Id) -> CommandResult {
        match self.find_node_idx(&id) {
            Some(idx) => {
                // delete all edges to or from this node
                let mut edges_touching: Vec<Id> = vec![];

                for edge in &self.edges {
                    if (&edge.from == id || &edge.to == id) && !edges_touching.contains(&&edge.id) {
                        edges_touching.push(edge.id.clone())
                    }
                }

                for delete in &edges_touching {
                    if let Some(idx) = self.find_edge_idx(delete) {
                        self.edges.remove(idx);
                    }
                }

                self.nodes.remove(idx);

                CommandResult::new(format!("node {} removed", id))
            }
            None => CommandResult::new(format!("node {} not found", id)),
        }
    }
}
