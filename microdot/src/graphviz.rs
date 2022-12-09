use crate::colors::{Color, ColorScheme, Colors};
use anyhow::anyhow;
use askama::Template;
use command_macros::cmd;
use hyphenation::{Language, Load, Standard};
use microdot_core::exporter::{Exporter, NodeHighlight};
use microdot_core::graph::Graph;
use microdot_core::hash::extract_hashtags;
use microdot_core::{Id, Label};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use std::process::Output;
use textwrap::wrap_algorithms::{wrap_optimal_fit, Penalties};
use textwrap::{fill, Options, WordSplitter};

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

#[derive(Copy, Clone)]
pub enum DisplayMode {
    Interactive,
    Presentation,
}

pub fn compile_dot(path: &Path, _display_mode: DisplayMode) -> Result<(), anyhow::Error> {
    if installed_graphviz_version().is_none() {
        return Err(anyhow::Error::msg("graphviz not installed"));
    }

    for ext in ["svg", "png"] {
        let out = path.with_extension(ext);

        let Output { status, stderr, .. } =
            cmd!(dot(path)(&format!("-T{}", ext))("-o")(out)).output()?;

        if !status.success() {
            let stderr = String::from_utf8_lossy(&stderr).to_string();
            return Err(anyhow!(stderr));
        }
    }

    Ok(())
}

pub struct GraphVizExporter {
    inner_content: String,
    is_left_right: bool,
    is_first_edge: bool,
    display_mode: DisplayMode,
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

        let wrapping_options = {
            let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
            let splitter = WordSplitter::Hyphenation(dictionary);
            Options::new(40).word_splitter(splitter)
        };

        let base_label = &label.to_string();

        let (hash_tags, label_text) = extract_hashtags(base_label);

        let id = match self.display_mode {
            DisplayMode::Interactive => id.to_string(),
            DisplayMode::Presentation => "".to_string(),
        };

        let label_text = fill(&label_text, &wrapping_options);

        let bgcolor = match highlight {
            NodeHighlight::Normal => Colors::white(),
            NodeHighlight::SearchResult => Color::from_rgb(208, 204, 204),
            NodeHighlight::CurrentNode => Colors::white(),
        };

        let hash_tags: Vec<_> = hash_tags
            .iter()
            .map(|tag| HashTagViewModel {
                label: tag.to_string(),
                bgcolor: ColorScheme::series(tag.hash()).get_fill_color(),
            })
            .collect();

        let colspan: usize = if hash_tags.is_empty() {
            1
        } else {
            hash_tags.len()
        };

        let label_vm = NodeHtmlLabelViewModel {
            id,
            label: label_text.clone(),
            label_wrapped: escape_label(&label_text),
            hash_tags,
            colspan,
            bgcolor,
        };

        let line = label_vm.render().unwrap();

        self.inner_content.push_str(&line);
        self.inner_content.push('\n');
    }

    fn add_edge(&mut self, id: &Id, from: &Id, to: &Id) {
        if self.is_first_edge {
            self.inner_content.push('\n');
            self.is_first_edge = false;
        }

        let edge_params = hashmap! {
            "id" => id.to_string(),
            "escaped_id" => escape_id(id.to_string()),
            "escaped_from" => escape_id(from.to_string()),
            "escaped_to" => escape_id(to.to_string()),
        };

        let line = match self.display_mode {
            DisplayMode::Interactive => template(
                r#"    ${escaped_from} -> ${escaped_to} [label=${escaped_id}];"#,
                &edge_params,
            ),
            DisplayMode::Presentation => {
                template(r#"    ${escaped_from} -> ${escaped_to};"#, &edge_params)
            }
        };

        self.inner_content.push_str(&line);
        self.inner_content.push('\n');
    }
}

impl GraphVizExporter {
    pub fn new(display_mode: DisplayMode) -> Self {
        Self {
            inner_content: "".into(),
            is_left_right: false,
            is_first_edge: true,
            display_mode,
        }
    }

    pub fn export(&mut self, graph: &Graph) -> String {
        graph.export(self);

        let rank_dir = if self.is_left_right { "LR" } else { "TB" };
        let rank_dir = rank_dir.to_string();
        let edge_color = ColorScheme::normal().get_stroke_color();

        let vm = GraphViewModel {
            rank_dir,
            edge_color,
            inner_content: self.inner_content.clone(),
        };
        vm.render().unwrap()
    }
}

fn escape_label(label: &str) -> String {
    format!("\"{}\"", label.replace('\n', "\\n").replace('"', "\\\""))
}

fn escape_id<S: Into<String>>(id: S) -> String {
    let id: String = id.into();
    format!("\"{}\"", id.replace('"', "\\\""))
}

#[allow(dead_code)]
fn prepare_label(label: &str, wrap: f64) -> String {
    let splitter = textwrap::WordSeparator::UnicodeBreakProperties;
    let fragments: Vec<_> = splitter.find_words(label).collect();
    let lines = match wrap_optimal_fit(&fragments, &[wrap], &Penalties::new()) {
        Ok(lines) => lines,
        Err(_) => return "<wrap failed>".into(),
    };
    let mut res = String::new();
    for line in &lines {
        let words = line.iter().map(|w| w.word).collect::<Vec<_>>().join(" ");
        res.push_str(&words);
        res.push('\n');
    }
    if !lines.is_empty() {
        res.truncate(res.len() - 1)
    }
    res
}

#[derive(Template)]
#[template(path = "node.html")]
#[allow(dead_code)]
struct NodeHtmlLabelViewModel {
    id: String,
    label: String,
    label_wrapped: String,
    colspan: usize,
    hash_tags: Vec<HashTagViewModel>,
    bgcolor: Color,
}

#[derive(Template)]
#[template(path = "hashtag.html")]
struct HashTagViewModel {
    label: String,
    bgcolor: Color,
}

#[derive(Template)]
#[template(path = "graph.txt")]
struct GraphViewModel {
    rank_dir: String,
    edge_color: Color,
    inner_content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{compile_input_string_content, git_root};

    #[test]
    fn runs_node_template() {
        let lines = vec![
            "this is the first",
            "line in the thing",
            "and here is a third",
        ];
        let label = NodeHtmlLabelViewModel {
            id: "n99".into(),
            label: lines.join("\n"),
            label_wrapped: escape_label(&lines.join("\n")),
            hash_tags: vec![
                HashTagViewModel {
                    bgcolor: Color::from_rgb(255, 0, 0),
                    label: "#hash1".into(),
                },
                HashTagViewModel {
                    bgcolor: Color::from_rgb(0, 255, 0),
                    label: "#hash2".into(),
                },
            ],
            colspan: 2,
            bgcolor: Colors::white(),
        };

        println!("{}", label.render().unwrap());
    }

    #[test]
    fn escapes_label() {
        assert_eq!(r#""abc""#, escape_label("abc"));
        assert_eq!(r#""a\"bc""#, escape_label(r#"a"bc"#));
        assert_eq!(r#""a\nbc""#, escape_label("a\nbc"));
    }

    #[test]
    fn prepares_label() {
        let instr =
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Cras ut egestas velit.";
        let outstr = r#"Lorem ipsum dolor sit amet,
consectetur adipiscing elit.
Cras ut egestas velit."#;

        assert_eq!(outstr, prepare_label(instr, 30.0f64));
    }

    #[test]
    fn test_graphviz_compiles() {
        let dot_file = dirs::home_dir()
            .unwrap()
            .join("src/github.com/stevecooperorg/microdot/test_data/exports_graph.dot");

        assert!(
            dot_file.exists(),
            "dot file does not exist: '{}'",
            dot_file.to_string_lossy()
        );

        let compile_result = compile_dot(&dot_file, DisplayMode::Interactive);
        assert!(compile_result.is_ok());
    }

    #[test]
    fn test_graphviz_compile_fellowship() {
        compile_input_string_content(git_root().unwrap().join("examples/fellowship.txt"));
    }

    #[test]
    fn test_graphviz_compile_business() {
        compile_input_string_content(git_root().unwrap().join("examples/business_example_1.txt"));
    }

    #[test]
    fn test_graphviz_compile_readme_example_1() {
        compile_input_string_content(git_root().unwrap().join("examples/readme_example_1.txt"));
    }

    #[test]
    fn test_graphviz_installed() {
        let version = installed_graphviz_version().expect("could not find graphviz version");
        let major_version = *version
            .split(".")
            .collect::<Vec<_>>()
            .first()
            .expect("could not find major version");
        let major_version: u32 = major_version.parse().expect("could not parse as number");

        assert!(
            major_version >= 3,
            "need a recent version of graphviz - have {}",
            major_version
        );
    }
}
