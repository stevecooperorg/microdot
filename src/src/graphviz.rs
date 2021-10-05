use crate::graph::Graph;
use crate::{Exporter, Id, Label};
use command_macros::cmd;
use hyphenation::{Language, Load, Standard};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use textwrap::word_separators::UnicodeBreakProperties;
use textwrap::wrap_algorithms::OptimalFit;
use textwrap::{fill, Options};

const JULEP: &str = "#73DBE6";
const PACIFICA: &str = "#2BBDCB";
const LEMONADE: &str = "#FFDD99";
const BRIGHT_SUN: &str = "#FFBB16";
const ATHENS: &str = "#F8F8FA";
const LINKWATER: &str = "#E6EBF8";
const GHOST: &str = "#DFE2EB";
const COMET: &str = "#485478";
const MARTINIQUE: &str = "#242D48";
const IRIS: &str = "#C882D9";
const ORCHID: &str = "#B25DC6";
const EMPIRE: &str = "#821499";
const RAIN: &str = "#A136B4";

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

pub fn installed_graphviz_version() -> Option<String> {
    // dot - graphviz version 2.49.1 (20210923.0004)
    let stderr = match cmd!(dot("-V")).output().ok() {
        Some(output) => output.stderr,
        None => return None,
    };
    let stderr = String::from_utf8_lossy(&stderr).to_string();
    let rx = Regex::new(r#"^dot - graphviz version (?P<ver>[0-9\.]+)"#).expect("not a valid rx");
    let caps = rx.captures(&stderr).map(|c| {
        c.name("ver")
            .expect("should have named group")
            .as_str()
            .into()
    });
    caps
}

pub fn compile_dot(path: &Path) -> Result<(), anyhow::Error> {
    if !installed_graphviz_version().is_some() {
        return Err(anyhow::Error::msg("graphviz not installed"));
    }

    let out = path.with_extension("svg");
    // dot "$(DEFAULT_DOT)" -Tsvg -o "$(DEFAULT_SVG)"
    cmd!(dot(path)("-Tsvg")("-o")(out)).output()?;

    Ok(())
}

pub struct GraphVizExporter {
    inner_content: String,
    debug_mode: bool,
    is_left_right: bool,
    is_first_edge: bool,
}

fn template(template_str: &str, variables: &HashMap<&str, String>) -> String {
    let mut result = template_str.to_string();
    for (k, v) in variables.iter() {
        let search_for = format!("${{{}}}", k);
        result = result.replace(&search_for, v);
    }
    result
}

struct ColorScheme {
    font_color: String,
    fill_color: String,
    stroke_color: String,
    node_border_width: f64,
    edge_border_width: f64,
}

impl ColorScheme {
    fn highlight() -> Self {
        Self {
            font_color: MARTINIQUE.to_string(),
            fill_color: IRIS.to_string(),
            stroke_color: ORCHID.to_string(),
            node_border_width: 3.0f64,
            edge_border_width: 3.0f64,
        }
    }

    fn normal() -> Self {
        Self {
            font_color: MARTINIQUE.to_string(),
            fill_color: JULEP.to_string(),
            stroke_color: PACIFICA.to_string(),
            node_border_width: 2.0f64,
            edge_border_width: 2.0f64,
        }
    }
}

impl Exporter for GraphVizExporter {
    fn set_direction(&mut self, is_left_right: bool) {
        self.is_left_right = is_left_right;
    }

    fn add_node(&mut self, id: &Id, label: &Label, highlight: bool) {
        // TODO: probably horrific perf.

        let wrapping_options: Options<OptimalFit, UnicodeBreakProperties, Standard> = {
            let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
            Options::new(40).word_splitter(dictionary)
        };

        let color_scheme = if highlight {
            ColorScheme::highlight()
        } else {
            ColorScheme::normal()
        };

        let ColorScheme {
            font_color,
            fill_color,
            stroke_color,
            node_border_width,
            edge_border_width,
        } = color_scheme;

        let label_text = if self.debug_mode {
            let unwrapped = format!("{}: {}", id.0, label.0);
            let wrapped = fill(&unwrapped, &wrapping_options);
            wrapped
        } else {
            label.0.clone()
        };

        let node_params = hashmap! {
            "id" => id.0.clone(),
            "label" => label.0.clone(),
            "escaped_id" => escape_id(&id.0),
            "label_text" => label_text,
            "stroke_color" => stroke_color,
            "fill_color" => fill_color,
            "font_color" => font_color,
            "width" => node_border_width.to_string()
        };

        const LINE_TEMPLATE: &str = r#"    ${escaped_id} [label="${label_text}" fillcolor="${fill_color}" color="${stroke_color}" fontcolor="${font_color}"]"#;

        let line = template(LINE_TEMPLATE, &node_params);

        self.inner_content.push_str(&line);
        self.inner_content.push_str("\n");
    }

    fn add_edge(&mut self, id: &Id, from: &Id, to: &Id) {
        if self.is_first_edge {
            self.inner_content.push_str("\n");
            self.is_first_edge = false;
        }

        let edge_params = hashmap! {
            "id" => id.0.clone(),
            "escaped_id" => escape_id(&id.0),
            "escaped_from" => escape_id(&from.0),
            "escaped_to" => escape_id(&to.0),
        };

        let line = if self.debug_mode {
            template(
                r#"    ${escaped_from} -> ${escaped_to} [label=${escaped_id}];"#,
                &edge_params,
            )
        } else {
            template(r#"    ${escaped_from} -> ${escaped_to};"#, &edge_params)
        };

        self.inner_content.push_str(&line);
        self.inner_content.push_str("\n");
    }
}

impl GraphVizExporter {
    pub fn new() -> Self {
        Self {
            inner_content: "".into(),
            debug_mode: true,
            is_left_right: false,
            is_first_edge: true,
        }
    }

    pub fn export(&mut self, graph: &Graph) -> String {
        let template = include_str!("template.dot");

        graph.export(self);

        let rankdir = if self.is_left_right { "LR" } else { "TB" };

        let content = template
            .replace("${RANKDIR}", rankdir)
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
    use super::*;
    use crate::graph::Graph;
    use crate::{GraphCommand, Id, Label};
    use std::path::Path;

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

        graph.apply_command(GraphCommand::InsertNode {
            label: Label::new("hij"),
        });

        graph.apply_command(GraphCommand::LinkEdge {
            from: Id::new("n0"),
            to: Id::new("n1"),
        });

        graph.apply_command(GraphCommand::LinkEdge {
            from: Id::new("n1"),
            to: Id::new("n2"),
        });

        graph.search(Label::new("abc"));

        let mut exporter = GraphVizExporter::new();

        let dot = exporter.export(&graph);

        assert_eq!(
            include_str!("../test_data/exports_graph.dot").to_string(),
            dot
        );
    }

    #[test]
    fn test_graphviz_compiles() {

        let dot_file = dirs::home_dir().unwrap().join("src/github.com/stevecooperorg/microdot/src/test_data/exports_graph.dot");

        assert!(
            dot_file.exists(),
            "dot file does not exist: '{}'",
            dot_file.to_string_lossy()
        );

        let compile_result = compile_dot(&dot_file);
        assert!(compile_result.is_ok());
    }

    #[test]
    fn test_graphviz_installed() {
        let version = installed_graphviz_version().expect("could not find graphviz version");
        let major_version = *version
            .split(".")
            .collect::<Vec<_>>()
            .first()
            .expect("could not find major version");

        assert_eq!(major_version, "2");
    }
}
