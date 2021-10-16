use crate::graph::Graph;
use crate::{Exporter, Id, Label, NodeHighlight};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct JsonExporter {
    nodes: Vec<Value>,
    edges: Vec<Value>,
    is_left_right: bool,
}

impl Exporter for JsonExporter {
    fn set_direction(&mut self, is_left_right: bool) {
        self.is_left_right = is_left_right;
    }

    fn add_node(&mut self, id: &Id, label: &Label, _highlight: NodeHighlight) {
        let node = json!({
            "id": id.0.clone(),
            "label": label.0.clone()
        });

        self.nodes.push(node);
    }

    fn add_edge(&mut self, _id: &Id, from: &Id, to: &Id) {
        let edge = json! { {
            "from": from.0.clone(),
            "to": to.0.clone()
        }};

        self.edges.push(edge);
    }
}

impl Default for JsonExporter {
    fn default() -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
            is_left_right: false,
        }
    }
}
impl JsonExporter {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn export(&mut self, graph: &Graph) -> String {
        graph.export(self);

        (json! {{
        "nodes": self.nodes,
        "edges": self.edges,
        "is_left_right": self.is_left_right
        }})
        .to_string()
    }
}

pub struct JsonImporter {
    content: String,
}

#[derive(Serialize, Deserialize)]
struct JsonNode {
    id: Id,
    label: Label,
}

#[derive(Serialize, Deserialize)]
struct JsonEdge {
    from: Id,
    to: Id,
}

#[derive(Serialize, Deserialize, Default)]
struct JsonGraph {
    nodes: Vec<JsonNode>,
    edges: Vec<JsonEdge>,
    is_left_right: bool,
}

pub fn empty_json_graph() -> String {
    let empty = JsonGraph::default();
    serde_json::to_string(&empty).expect("should be infallible")
}

impl JsonImporter {
    pub fn new<S: Into<String>>(content: S) -> Self {
        JsonImporter {
            content: content.into(),
        }
    }

    pub fn import(&self) -> Result<Graph, anyhow::Error> {
        let value: JsonGraph = serde_json::from_str(&self.content)?;

        let mut translate = HashMap::new();
        let mut graph = Graph::new();

        graph.set_direction(value.is_left_right);

        for node in &value.nodes {
            let (new_id, _) = graph.insert_node(node.label.clone());
            translate.insert(node.id.clone(), new_id);
        }

        for edge in &value.edges {
            let new_from_id = translate.get(&edge.from);
            let new_to_id = translate.get(&edge.to);
            if let (Some(new_from_id), Some(new_to_id)) = (new_from_id, new_to_id) {
                graph.link_edge(new_from_id, new_to_id);
            }
        }

        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GraphCommand;

    #[test]
    fn imports_graph() {
        let content = include_str!("../test_data/imports_graph.json").to_string();
        let importer = JsonImporter::new(content);
        importer.import().expect("could not import");
    }

    #[test]
    fn round_trips_graph() {
        let content = include_str!("../test_data/imports_graph.json").to_string();
        let importer = JsonImporter::new(content.clone());
        let graph = importer.import().expect("could not import");
        let mut exporter = JsonExporter::new();
        let exported = exporter.export(&graph);
        assert_eq!(
            content, exported,
            "round-trip should have lost or changed nothing"
        );
    }

    #[test]
    fn creates_empty_graph() {
        assert_eq!(
            empty_json_graph(),
            r#"{"nodes":[],"edges":[],"is_left_right":false}"#.to_string()
        );
    }

    #[test]
    fn exports_graph() {
        let mut graph = Graph::new();

        graph.apply_command(GraphCommand::InsertNode {
            label: Label::new("abc"),
        });

        graph.apply_command(GraphCommand::InsertNode {
            label: Label::new("def"),
        });

        graph.apply_command(GraphCommand::LinkEdge {
            from: Id::new("n0"),
            to: Id::new("n1"),
        });

        let mut exporter = JsonExporter::new();
        let exported = exporter.export(&graph);

        assert_eq!(
            include_str!("../test_data/exports_graph.json").to_string(),
            exported
        );
    }
}
