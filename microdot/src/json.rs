use anyhow::Result;
use microdot_core::exporter::{Exporter, NodeHighlight};
use microdot_core::graph::Graph;
use microdot_core::{Direction, Id, Label};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Default)]
pub struct JsonExporter {
    nodes: Vec<Value>,
    edges: Vec<Value>,
    direction: Direction,
}

impl Exporter for JsonExporter {
    fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    fn add_node(&mut self, id: &Id, label: &Label, _highlight: NodeHighlight) {
        let node = json!({
            "id": id.to_string(),
            "label": label.to_string()
        });

        self.nodes.push(node);
    }

    fn add_edge(&mut self, _id: &Id, from: &Id, to: &Id) {
        let edge = json! { {
            "from": from.to_string(),
            "to": to.to_string()
        }};

        self.edges.push(edge);
    }
}

impl JsonExporter {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn export_json(&mut self, graph: &Graph) -> String {
        graph.export(self);

        let value = json! {{
        "nodes": self.nodes,
        "edges": self.edges,
        "direction": self.direction
        }};
        serde_json::to_string_pretty(&value).expect("could not serialise json")
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

#[derive(Serialize, Default)]
struct JsonGraph {
    nodes: Vec<JsonNode>,
    edges: Vec<JsonEdge>,
    direction: Direction,
}

impl<'de> Deserialize<'de> for JsonGraph {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct JsonGraphHelper {
            nodes: Vec<JsonNode>,
            edges: Vec<JsonEdge>,
            #[serde(default)]
            is_left_right: Option<bool>,
            #[serde(default)]
            direction: Option<Direction>,
        }

        let helper = JsonGraphHelper::deserialize(deserializer)?;

        let direction = match (helper.is_left_right, helper.direction) {
            // New format: direction is present
            (_, Some(dir)) => dir,
            // Old format: is_left_right is present
            (Some(is_lr), None) => {
                if is_lr {
                    Direction::LR
                } else {
                    Direction::TB
                }
            },
            // Neither present: default to TB
            (None, None) => Direction::TB,
        };

        Ok(JsonGraph {
            nodes: helper.nodes,
            edges: helper.edges,
            direction,
        })
    }
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

        graph.set_direction(value.direction);

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

    pub fn load(json_file: impl AsRef<Path>) -> Result<Graph> {
        let json_file = json_file.as_ref();
        let json_content = {
            let mut f = File::open(json_file)?;
            let mut s = "".to_string();
            f.read_to_string(&mut s)?;
            s
        };

        let importer = JsonImporter::new(json_content);
        let graph = importer.import()?;
        println!("Loaded graph from {}: {}", json_file.display(), graph);
        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use microdot_core::command::GraphCommand;
    use microdot_core::graph::Graph;

    #[test]
    fn imports_graph() {
        let content = include_str!("../../test_data/imports_graph.json").to_string();
        let importer = JsonImporter::new(content);
        importer.import().expect("could not import");
    }

    #[test]
    fn round_trips_graph() {
        let content = include_str!("../../test_data/imports_graph.json").to_string();
        let importer = JsonImporter::new(content.clone());
        let graph = importer.import().expect("could not import");
        let mut exporter = JsonExporter::new();
        let exported = exporter.export_json(&graph);
        // After round-trip, old format should be upgraded to new format
        let parsed: Value = serde_json::from_str(&exported).expect("should parse");
        assert_eq!(
            parsed["direction"],
            "LR",
            "round-trip should upgrade old format to new format with direction: LR"
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
        let exported = exporter.export_json(&graph);

        // After upgrade, exports should use direction enum instead of is_left_right
        let parsed: Value = serde_json::from_str(&exported).expect("should parse");
        assert_eq!(
            parsed["direction"],
            "TB",
            "exports should use new direction format: TB"
        );
    }

    #[test]
    fn deserializes_old_format_with_is_left_right_true() {
        let content = r#"{
            "nodes": [{"id": "n0", "label": "abc"}],
            "edges": [],
            "is_left_right": true
        }"#;
        let importer = JsonImporter::new(content);
        let graph = importer.import().expect("could not import");
        // Should convert is_left_right: true to Direction::LR
        let mut exporter = JsonExporter::new();
        let exported = exporter.export_json(&graph);
        let parsed: Value = serde_json::from_str(&exported).expect("should parse");
        assert_eq!(
            parsed["direction"],
            "LR",
            "old format is_left_right: true should deserialize to LR"
        );
    }

    #[test]
    fn deserializes_old_format_with_is_left_right_false() {
        let content = r#"{
            "nodes": [{"id": "n0", "label": "abc"}],
            "edges": [],
            "is_left_right": false
        }"#;
        let importer = JsonImporter::new(content);
        let graph = importer.import().expect("could not import");
        // Should convert is_left_right: false to Direction::TB
        let mut exporter = JsonExporter::new();
        let exported = exporter.export_json(&graph);
        let parsed: Value = serde_json::from_str(&exported).expect("should parse");
        assert_eq!(
            parsed["direction"],
            "TB",
            "old format is_left_right: false should deserialize to TB"
        );
    }

    #[test]
    fn deserializes_new_format_with_direction() {
        let content = r#"{
            "nodes": [{"id": "n0", "label": "abc"}],
            "edges": [],
            "direction": "RL"
        }"#;
        let importer = JsonImporter::new(content);
        let graph = importer.import().expect("could not import");
        // Direction is now preserved through the entire chain
        let mut exporter = JsonExporter::new();
        let exported = exporter.export_json(&graph);
        let parsed: Value = serde_json::from_str(&exported).expect("should parse");
        assert_eq!(
            parsed["direction"],
            "RL",
            "new format should preserve direction: RL"
        );
    }

    #[test]
    fn deserializes_all_direction_variants() {
        for (direction_str, expected_bool) in [("LR", true), ("RL", true), ("TB", false), ("BT", false)] {
            let content = format!(
                r#"{{
                    "nodes": [{{"id": "n0", "label": "abc"}}],
                    "edges": [],
                    "direction": "{}"
                }}"#,
                direction_str
            );
            let importer = JsonImporter::new(&content);
            let graph = importer.import().expect(&format!("could not import direction: {}", direction_str));
            // Verify conversion to bool is correct
            let mut exporter = JsonExporter::new();
            graph.export(&mut exporter);
            assert_eq!(
                exporter.direction.to_bool(),
                expected_bool,
                "direction {} should convert to bool {}",
                direction_str,
                expected_bool
            );
        }
    }
}
