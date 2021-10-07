use crate::graph::Graph;
use crate::{Exporter, Id, Label, NodeHighlight};
use command_macros::cmd;
use hyphenation::{Language, Load, Standard};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use textwrap::word_separators::UnicodeBreakProperties;
use textwrap::wrap_algorithms::OptimalFit;
use textwrap::{fill, Options};

// teals
const JULEP: &str = "#73DBE6";
const PACIFICA: &str = "#2BBDCB";
const PEACOCK: &str = "#01828E";

// yellows
const LEMONADE: &str = "#FFDD99";
const BRIGHT_SUN: &str = "#FFBB16";
const BUMBLEBEE: &str = "E99823";
const TUSCAN: &str = "CC851F";

// greys
const ATHENS: &str = "#F8F8FA";
const LINKWATER: &str = "#E6EBF8";
const GHOST: &str = "#DFE2EB";
const COMET: &str = "#485478";

// ultra-dark blue
const MARTINIQUE: &str = "#242D48";

// purples
const IRIS: &str = "#C882D9";
const ORCHID: &str = "#B25DC6";
const RAIN: &str = "#A136B4";
const EMPIRE: &str = "#821499";

struct ColorScheme {
    font_color: String,
    fill_color: String,
    stroke_color: String,
    node_border_width: f64,
}

impl ColorScheme {
    fn search_result() -> Self {
        Self {
            font_color: MARTINIQUE.to_string(),
            fill_color: JULEP.to_string(),
            stroke_color: PEACOCK.to_string(),
            node_border_width: 3.0f64,
        }
    }

    fn current() -> Self {
        Self {
            font_color: MARTINIQUE.to_string(),
            fill_color: LEMONADE.to_string(),
            stroke_color: TUSCAN.to_string(),
            node_border_width: 3.0f64,
        }
    }

    fn normal() -> Self {
        Self {
            font_color: MARTINIQUE.to_string(),
            fill_color: ATHENS.to_string(),
            stroke_color: COMET.to_string(),
            node_border_width: 2.0f64,
        }
    }
}

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

impl Exporter for GraphVizExporter {
    fn set_direction(&mut self, is_left_right: bool) {
        self.is_left_right = is_left_right;
    }

    fn add_node(&mut self, id: &Id, label: &Label, highlight: NodeHighlight) {
        // TODO: probably horrific perf.

        let wrapping_options: Options<OptimalFit, UnicodeBreakProperties, Standard> = {
            let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
            Options::new(40).word_splitter(dictionary)
        };

        let color_scheme = match highlight {
            NodeHighlight::Normal => ColorScheme::normal(),
            NodeHighlight::SearchResult => ColorScheme::search_result(),
            NodeHighlight::CurrentNode => ColorScheme::current(),
        };

        let ColorScheme {
            font_color,
            fill_color,
            stroke_color,
            node_border_width,
            ..
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
            "label_text" => escape_label(&label_text),
            "stroke_color" => stroke_color,
            "fill_color" => fill_color,
            "font_color" => font_color,
            "width" => node_border_width.to_string()
        };

        const LINE_TEMPLATE: &str = r#"    ${escaped_id} [label=${label_text} fillcolor="${fill_color}" color="${stroke_color}" fontcolor="${font_color}"]"#;

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

        let edge_color = ColorScheme::normal().stroke_color;

        let content = template
            .replace("${RANKDIR}", rankdir)
            .replace("${EDGECOLOR}", &edge_color)
            .replace("${INNER_CONTENT}", &self.inner_content);

        content
    }
}

fn escape_label(label: &str) -> String {
    format!("\"{}\"", label.replace("\n", "\\n").replace("\"", "\\\""))
}

fn escape_id(id: &str) -> String {
    format!("\"{}\"", id.replace("\"", "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    use crate::parser::parse_line;
    use crate::repl::repl;
    use crate::{Command, GraphCommand, Id, Interaction, Label, Line};
    use std::collections::VecDeque;
    use std::path::{Path, PathBuf};

    #[test]
    fn escapes_label() {
        assert_eq!(r#""abc""#, escape_label("abc"));
        assert_eq!(r#""a\"bc""#, escape_label(r#"a"bc"#));
        assert_eq!(r#""a\nbc""#, escape_label("a\nbc"));
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
        let dot_file = dirs::home_dir()
            .unwrap()
            .join("src/github.com/stevecooperorg/microdot/src/test_data/exports_graph.dot");

        assert!(
            dot_file.exists(),
            "dot file does not exist: '{}'",
            dot_file.to_string_lossy()
        );

        let compile_result = compile_dot(&dot_file);
        assert!(compile_result.is_ok());
    }

    fn git_root() -> PathBuf {
        dirs::home_dir()
            .unwrap()
            .join("src/github.com/stevecooperorg/microdot")
    }

    #[test]
    fn test_graphviz_compile_fellowship() {
        compile_input_string_content(git_root().join("examples/fellowship.txt"));
    }

    #[test]
    fn test_graphviz_compile_readme_example_1() {
        compile_input_string_content(git_root().join("examples/readme_example_1.txt"));
    }

    struct AutoInteraction {
        lines: VecDeque<String>,
        log: String,
    }

    impl AutoInteraction {
        fn new(lines: VecDeque<String>) -> Self {
            Self {
                lines,
                log: Default::default(),
            }
        }

        fn log(&self) -> String {
            self.log.clone()
        }
    }

    impl Interaction for AutoInteraction {
        fn read(&mut self, prompt: &str) -> rustyline::Result<String> {
            match self.lines.pop_front() {
                Some(line) => Ok(line),
                None => Err(rustyline::error::ReadlineError::Eof),
            }
        }

        fn add_history<S: AsRef<str> + Into<String>>(&mut self, history: S) -> bool {
            self.log.push_str(">> ");
            self.log.push_str(&history.into());
            self.log.push_str("\n");
            true
        }

        fn log<S: AsRef<str> + Into<String>>(&mut self, message: S) {
            self.log.push_str(&message.into());
            self.log.push_str("\n");
        }
    }

    fn compile_input_string_content(text_file: PathBuf) {
        assert!(
            text_file.exists(),
            "text file does not exist: '{}'",
            text_file.to_string_lossy()
        );

        // read the file as lines and run it through the repl;
        let mut graph = Graph::new();

        let text_content = std::fs::read_to_string(&text_file).expect("could not read file");
        let lines: VecDeque<_> = text_content.lines().map(|l| l.to_string()).collect();
        let mut auto_interaction = AutoInteraction::new(lines);
        repl(
            &mut auto_interaction,
            &text_file.with_extension("json"),
            &mut graph,
        );

        let mut exporter = GraphVizExporter::new();
        let exported = exporter.export(&graph);

        let dot_file = text_file.with_extension("dot");
        std::fs::write(&dot_file, exported).expect("could not write dot file");

        let log_file = text_file.with_extension("log");
        std::fs::write(&log_file, auto_interaction.log()).expect("could not write log file");

        compile_dot(&dot_file).expect(&format!(
            "Could not compile '{}'",
            dot_file.to_string_lossy()
        ));
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
