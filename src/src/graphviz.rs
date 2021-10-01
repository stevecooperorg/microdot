use crate::graph::Graph;
use crate::{Exporter, Id, Label};

pub struct GraphVizExporter {
    inner_content: String,
}

impl Exporter for GraphVizExporter {
    fn add_node(&mut self, id: &Id, label: &Label) {
        let line = format!(
            "    {} [label={}];\n",
            escape_id(&id.0),
            escape_label(&label.0)
        );
        self.inner_content.push_str(&line);
    }

    fn add_edge(&mut self, from: &Id, to: &Id) {
        let line = format!("    {} -> {};\n", escape_id(&from.0), escape_id(&to.0));
        self.inner_content.push_str(&line);
    }
}

impl GraphVizExporter {
    pub fn new() -> Self {
        Self {
            inner_content: "".into(),
        }
    }

    pub fn export(&mut self, graph: &Graph) -> String {
        let template = include_str!("template.dot");
        graph.export(self);
        let content = template.replace("${INNER_CONTENT}", &self.inner_content);
        content
    }
}

fn escape_label(label: &str) -> String {
    format!("\"{}\"", label.replace("\"", "\\\""))
}

fn escape_id(id: &str) -> String {
    format!("\"{}\"", id.replace("\"", "\\\""))
}

#[cfg(test)]
mod tests {
    use crate::graph::Graph;
    use crate::graphviz::{escape_label, GraphVizExporter};
    use crate::{GraphCommand, Id, Label};

    #[test]
    fn escapes_label() {
        assert_eq!(r#""abc""#, escape_label("abc"));
        assert_eq!(r#""a\"bc""#, escape_label("a\"bc"));
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
        let mut exporter = GraphVizExporter::new();
        let dot = exporter.export(&graph);
        assert_eq!(
            include_str!("../test_data/exports_graph.dot").to_string(),
            dot
        );
    }
}
