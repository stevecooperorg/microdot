use crate::graph::Graph;
use crate::{Exporter, Id, Label};
use hyphenation::{Language, Load, Standard};
use std::collections::HashMap;
use textwrap::word_separators::UnicodeBreakProperties;
use textwrap::wrap_algorithms::OptimalFit;
use textwrap::{fill, Options};

macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = hashmap!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}

fn colors() -> HashMap<String, String> {
    hashmap! {
      "julep".to_string() => "#73DBE6".to_string(),
      "pacifica".to_string() => "#2BBDCB".to_string(),
      "lemonade".to_string() => "#FFDD99".to_string(),
      "bright_sun".to_string() => "#FFBB16".to_string(),
      "athens".to_string() => "#F8F8FA".to_string(),
      "linkwater".to_string() => "#E6EBF8".to_string(),
      "ghost".to_string() => "#DFE2EB".to_string(),
      "comet".to_string() => "#485478".to_string(),
      "martinique".to_string() => "#242D48".to_string(),
      "iris".to_string() => "#C882D9".to_string(),
      "orchid".to_string() => "#B25DC6".to_string(),
      "empire".to_string() => "#821499".to_string(),
      "rain".to_string() => "#A136B4".to_string(),
    }
}

pub struct GraphVizExporter {
    inner_content: String,
    debug_mode: bool,
}

impl Exporter for GraphVizExporter {
    fn add_node(&mut self, id: &Id, label: &Label) {
        // TODO: probably horrific perf.

        let wrapping_options: Options<OptimalFit, UnicodeBreakProperties, Standard> = {
            let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
            Options::new(50).word_splitter(dictionary)
        };

        let label_text = if self.debug_mode {
            let unwrapped = format!("{}: {}", id.0, label.0);
            let wrapped = fill(&unwrapped, &wrapping_options);
            wrapped
        } else {
            label.0.clone()
        };
        let line = format!(
            "    {} [label={}];\n",
            escape_id(&id.0),
            escape_label(&label_text)
        );
        self.inner_content.push_str(&line);
    }

    fn add_edge(&mut self, id: &Id, from: &Id, to: &Id) {
        let edge_label = if self.debug_mode {
            format!(" [label={}]", id.0)
        } else {
            "".to_string()
        };
        let line = format!(
            "    {} -> {}{};\n",
            escape_id(&from.0),
            escape_id(&to.0),
            edge_label
        );
        self.inner_content.push_str(&line);
    }
}

impl GraphVizExporter {
    pub fn new() -> Self {
        Self {
            inner_content: "".into(),
            debug_mode: true,
        }
    }

    pub fn export(&mut self, graph: &Graph) -> String {
        let template = include_str!("template.dot");

        graph.export(self);

        let colors = colors();

        let content = template
            .replace("${NODE_COLOR}", colors.get("lemonade").unwrap())
            .replace("${NODE_FONT_COLOR}", colors.get("martinique").unwrap())
            .replace("${INNER_CONTENT}", &self.inner_content);

        content
    }
}

fn escape_label(label: &str) -> String {
    format!("\"{}\"", label.replace("\n", "\\n ").replace("\"", "\\\""))
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
