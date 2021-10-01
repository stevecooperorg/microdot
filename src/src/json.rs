use serde_json::Value;
use serde_json::json;
use crate::graph::Graph;
use crate::{Exporter, Id, Label};

pub struct JsonExporter {
    nodes: Vec<Value>,
    edges: Vec<Value>
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
        let edge = json!{ {
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
            edges: vec![]
        }
    }

    pub fn export(&mut self, graph: &Graph) -> String {
        graph.export(self);
        let content = json! {{
            "nodes": self.nodes,
            "edges": self.edges
            }}.to_string();
        content
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::Graph;
    use crate::{GraphCommand, Id, Label};
    use super::*;

    #[test]
    fn exports_graph() {
        let mut graph = Graph::new();
        graph.apply_command(GraphCommand::InsertNode { label: Label::new("abc") });
        graph.apply_command(GraphCommand::InsertNode { label: Label::new("def") });
        graph.apply_command(GraphCommand::LinkEdge {
            from: Id::new("n0"),
            to: Id::new("n1")
        });
        let mut exporter = JsonExporter::new();
        let dot = exporter.export(&graph);
        assert_eq!(include_str!("../test_data/exports_graph.json").to_string(), dot);
    }
}