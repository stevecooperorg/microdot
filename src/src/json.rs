use crate::graph::Graph;
use crate::{Exporter, GraphCommand, Id, Importer, Label};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;
use std::path::Path;

pub struct JsonExporter {
    nodes: Vec<Value>,
    edges: Vec<Value>,
}

impl Exporter for JsonExporter {
    fn add_node(&mut self, id: &Id, label: &Label) {
        let node = json!({
            "id": id.0.clone(),
            "label": label.0.clone()
        });

        self.nodes.push(node);
    }

    fn add_edge(&mut self, from: &Id, to: &Id) {
        let edge = json! { {
            "from": from.0.clone(),
            "to": to.0.clone()
        }};

        self.edges.push(edge);
    }
}

impl JsonExporter {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
        }
    }

    pub fn export(&mut self, graph: &Graph) -> String {
        graph.export(self);

        let content = json! {{
        "nodes": self.nodes,
        "edges": self.edges
        }}
        .to_string();

        content
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

#[derive(Serialize, Deserialize)]
struct JsonGraph {
    nodes: Vec<JsonNode>,
    edges: Vec<JsonEdge>,
}

impl JsonImporter {
    pub fn new<S: Into<String>>(content: S) -> Self {
        JsonImporter {
            content: content.into(),
        }
    }

    pub fn import(&self) -> Result<Graph, anyhow::Error> {
        let value: JsonGraph = serde_json::from_str(&self.content)?;

        let mut graph = Graph::new();

        for node in &value.nodes {
            unimplemented!("how should I deal with internal IDs?");

            graph.apply_command(GraphCommand::InsertNode { label })
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::graph::Graph;
    use crate::{GraphCommand, Id, Label};

    #[test]
    fn imports_graph() {
        let content = include_str!("../test_data/exports_graph.json").to_string();

        let importer = JsonImporter::new(content);

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

        let dot = exporter.export(&graph);

        assert_eq!(
            include_str!("../test_data/exports_graph.json").to_string(),
            dot
        );
    }
}
