use crate::hashmap;
use crate::util::write_if_different;
use anyhow::{anyhow, Context, Result};
use askama::Template;
use command_macros::cmd;
use hyphenation::{Language, Load, Standard};
use microdot_colors::colors::{Color, ColorScheme, Colors};
use microdot_core::exporter::{Exporter, NodeHighlight};
use microdot_core::graph::Graph;
use microdot_core::hash::HashTag;
use microdot_core::labels::NodeInfo;
use microdot_core::util::generate_hash;
use microdot_core::{Id, Label};
use once_cell::sync::OnceCell;
use regex::Regex;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use textwrap::wrap_algorithms::{wrap_optimal_fit, Penalties};
use textwrap::{fill, Options, WordSplitter};

#[derive(Copy, Clone)]
pub enum DisplayMode {
    Interactive,
    Presentation,
}

pub fn compile(path: &Path) -> Result<()> {
    let input_str = std::fs::read_to_string(path)?;
    let out_file = path.with_extension("svg");
    let html_file = path.with_extension("html");
    let image_url = out_file.file_name().unwrap().to_string_lossy().to_string();
    let image_title = out_file.file_stem().unwrap().to_string_lossy().to_string();

    dot::DotCompiler::compile_dot_str(input_str)
        .and_then(|string| {
            write_if_different(out_file, string)?;
            Ok(())
        })
        .and_then(|_| {
            let html = ImagePage {
                image_title,
                image_url,
            };
            let html_content = html.render()?;
            write_if_different(html_file, html_content)?;
            Ok(())
        })
}

#[derive(Template)]
#[template(path = "image_page.html")]
pub struct ImagePage {
    image_title: String,
    image_url: String,
}

impl ImagePage {
    pub fn new(image_title: impl Into<String>, image_url: impl Into<String>) -> Self {
        Self {
            image_title: image_title.into(),
            image_url: image_url.into(),
        }
    }
}

pub struct GraphVizExporter {
    nodes: Vec<NodeHtmlLabelViewModel>,
    subgraphs: BTreeMap<HashTag, Vec<NodeHtmlLabelViewModel>>,
    edges: Vec<EdgeViewModel>,
    is_left_right: bool,
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

        let wrap_size = if self.is_left_right { 40 } else { 25 };
        let wrapping_options = {
            let mut dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
            dictionary.minima = (3, 3);
            let splitter = WordSplitter::Hyphenation(dictionary);
            Options::new(wrap_size).word_splitter(splitter)
        };

        let NodeInfo {
            label: label_text,
            tags,
            variables,
            subgraph,
        } = NodeInfo::parse(label);

        let id = match self.display_mode {
            DisplayMode::Interactive => id.to_string(),
            DisplayMode::Presentation => "".to_string(),
        };

        let label_text = fill(&label_text, wrapping_options);

        let bgcolor = match highlight {
            NodeHighlight::Normal => Colors::white(),
            NodeHighlight::SearchResult => Color::from_rgb(208, 204, 204),
            NodeHighlight::CurrentNode => Colors::white(),
        };

        let mut hash_tags: Vec<_> = vec![];
        for tag in &tags {
            let label = tag.to_string();
            let bgcolor = tag_adjust(ColorScheme::series(tag.hash()).get_fill_color());
            let model = HashTagViewModel { label, bgcolor };
            hash_tags.push(model);
        }

        for var in variables.iter() {
            let label = format!("{}", var);
            let bgcolor =
                tag_adjust(ColorScheme::series(generate_hash(&var.name)).get_fill_color());
            let model = HashTagViewModel { label, bgcolor };
            hash_tags.push(model);
        }

        let colspan: usize = if hash_tags.is_empty() {
            1
        } else {
            hash_tags.len()
        };

        let label_vm = NodeHtmlLabelViewModel {
            id,
            label: escape_label(&label_text),
            label_wrapped: to_dot_label_string(&label_text),
            hash_tags,
            colspan,
            bgcolor,
        };

        let target = match subgraph {
            Some(subgraph_id) => self.subgraphs.entry(subgraph_id).or_default(),
            None => &mut self.nodes,
        };

        target.push(label_vm);
    }

    fn add_edge(&mut self, id: &Id, from: &Id, to: &Id) {
        let edge_vm = EdgeViewModel {
            display_mode: self.display_mode,
            id: id.clone(),
            from: from.clone(),
            to: to.clone(),
        };

        self.edges.push(edge_vm);
    }
}

struct EdgeViewModel {
    display_mode: DisplayMode,
    id: Id,
    from: Id,
    to: Id,
}

impl Display for EdgeViewModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} -> {}", self.id, self.from, self.to)
    }
}

impl Template for EdgeViewModel {
    fn render_into(&self, writer: &mut (impl std::fmt::Write + ?Sized)) -> askama::Result<()> {
        let edge_params = hashmap! {
            "id" => self.id.to_string(),
            "escaped_id" => escape_id(self.id.to_string()),
            "escaped_from" => escape_id(self.from.to_string()),
            "escaped_to" => escape_id(self.to.to_string()),
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

        writer.write_str(&line)?;
        Ok(())
    }

    const EXTENSION: Option<&'static str> = None;
    const SIZE_HINT: usize = 0;
    const MIME_TYPE: &'static str = "";
}

impl GraphVizExporter {
    pub fn new(display_mode: DisplayMode) -> Self {
        Self {
            is_left_right: false,
            nodes: Default::default(),
            edges: Default::default(),
            subgraphs: Default::default(),
            display_mode,
        }
    }

    fn build(&self) -> String {
        let mut built = String::new();
        for node in &self.nodes {
            let line = node.render().unwrap();
            built.push_str(&line);
            built.push('\n');
        }

        for (subgraph_id, nodes) in self.subgraphs.iter() {
            let bgcolor = subgraph_adjust(ColorScheme::series(subgraph_id.hash()).get_fill_color());
            let color = Color::from_rgb(255, 255, 255);
            built.push_str(&format!(
                "  subgraph cluster_{} {{\n",
                subgraph_id.to_string().replace('#', "")
            ));
            built.push_str(&format!("  label=\"{}\"", subgraph_id));
            built.push('\n');
            built.push_str(&format!("  bgcolor=\"{}\"", bgcolor));
            built.push('\n');
            built.push_str(&format!("  fontcolor=\"{}\"", color));
            built.push('\n');
            for node in nodes {
                let line = node.render().unwrap();
                built.push_str(&line);
                built.push('\n');
            }
            built.push_str("  }\n");
        }

        built.push('\n');

        for edge in &self.edges {
            let line = edge.render().unwrap();
            built.push_str(&line);
            built.push('\n');
        }

        built
    }

    pub fn export_dot(&mut self, graph: &Graph) -> String {
        graph.export(self);

        let rank_dir = if self.is_left_right { "LR" } else { "TB" };
        let rank_dir = rank_dir.to_string();
        let edge_color = ColorScheme::normal().get_stroke_color();
        let inner_content = self.build();

        let width = if self.is_left_right { 4.0f32 } else { 2.5f32 };
        let vm = GraphViewModel {
            rank_dir,
            edge_color,
            inner_content,
            width,
        };
        vm.render().unwrap()
    }
}

fn to_dot_label_string(label: &str) -> String {
    format!("\"{}\"", label.replace('\n', "\\n").replace('"', "\\\""))
}

fn tag_adjust(color: Color) -> Color {
    color.mix(Colors::white()).mute(1.0f64, 0.9f64)
}

fn subgraph_adjust(color: Color) -> Color {
    color.mute(1.0f64, 0.6f64)
}

fn escape_label(label: &str) -> String {
    label
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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
    width: f32,
}

mod dot {
    use super::*;

    pub fn installed_graphviz_version() -> Option<String> {
        static INSTANCE: OnceCell<Option<String>> = OnceCell::new();
        INSTANCE
            .get_or_init(installed_graphviz_version_inner)
            .clone()
    }

    fn installed_graphviz_version_inner() -> Option<String> {
        // dot - graphviz version 2.49.1 (20210923.0004)
        let stderr = match cmd!(dot("-V")).output().ok() {
            Some(output) => output.stderr,
            None => return None,
        };
        let stderr = String::from_utf8_lossy(&stderr).to_string();
        let rx = Regex::new(r"^dot - graphviz version (?P<ver>[0-9\.]+)").expect("not a valid rx");
        let caps = rx.captures(&stderr).map(|c| {
            c.name("ver")
                .expect("should have named group")
                .as_str()
                .into()
        });
        caps
    }

    pub struct DotCompiler {}

    impl DotCompiler {
        pub fn compile_dot_str<S: AsRef<str>>(input: S) -> Result<String> {
            if installed_graphviz_version().is_none() {
                return Err(anyhow::Error::msg("graphviz not installed"));
            }

            let mut child = Command::new("dot")
                .arg(format!("-T{}", "svg"))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            let child_stdin = child.stdin.as_mut().unwrap();
            child_stdin.write_all(input.as_ref().as_bytes())?;

            let output = child.wait_with_output()?;

            let Output {
                status,
                stderr,
                stdout,
            } = output;

            if status.success() {
                let stdout = std::str::from_utf8(&stdout)
                    .context("converting graphviz output to utf8")?
                    .to_string();
                Ok(stdout)
            } else {
                let stderr = String::from_utf8_lossy(&stderr).to_string();
                Err(anyhow!(stderr))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::JsonImporter;
    use crate::util::{compile_input_string_content, git_root};

    #[test]
    fn runs_node_template() {
        let lines = [
            "this is the first",
            "line in the thing",
            "and here is a third",
        ];
        let label = NodeHtmlLabelViewModel {
            id: "n99".into(),
            label: lines.join("\n"),
            label_wrapped: to_dot_label_string(&lines.join("\n")),
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
    fn converts_to_dot_label_string() {
        assert_eq!(r#""abc""#, to_dot_label_string("abc"));
        assert_eq!(r#""a\"bc""#, to_dot_label_string(r#"a"bc"#));
        assert_eq!(r#""a\nbc""#, to_dot_label_string("a\nbc"));
    }

    #[test]
    fn escapes_html_codes_in_label() {
        assert_eq!(r#"abc"#, escape_label("abc"));
        assert_eq!(r#"foo &lt;bar&gt;"#, escape_label(r#"foo <bar>"#));
        assert_eq!(r#"a&amp;b"#, escape_label("a&b"));
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

        let compile_result = compile(&dot_file);
        assert!(compile_result.is_ok());
    }

    #[test]
    fn test_graphviz_compile_fellowship() {
        compile_input_string_content(git_root().unwrap().join("examples/fellowship.txt"));
    }

    #[test]
    fn test_graphviz_compile_subgraphs() {
        compile_input_string_content(git_root().unwrap().join("examples/subgraphs.txt"));
    }

    #[test]
    fn test_graphviz_compile_variables() {
        compile_input_string_content(git_root().unwrap().join("examples/variables.txt"));
    }

    #[test]
    fn test_graphviz_compile_bit_of_everything() {
        compile_input_string_content(git_root().unwrap().join("examples/bit_of_everything.txt"));
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
        let version = dot::installed_graphviz_version().expect("could not find graphviz version");
        let major_version = *version
            .split('.')
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
    #[test]
    fn test_from_env_var() {
        let json_file = std::env::var("MICRODOT_EXTERNAL_JSON_FILE").unwrap_or_default();

        if json_file.is_empty() {
            return;
        }

        let json_file = Path::new(&json_file);
        if !json_file.exists() {
            panic!("json file does not exist: {}", json_file.to_string_lossy());
        }

        let graph = JsonImporter::load(json_file).expect("could not load json file");

        // export the graph to a dot string
        let mut dot_exporter = GraphVizExporter::new(DisplayMode::Interactive);
        let dot = dot_exporter.export_dot(&graph);

        let _ = dot::DotCompiler::compile_dot_str(dot).and_then(|string| {
            let out_file = json_file.with_extension("svg");
            write_if_different(out_file, string)?;
            Ok(())
        });
    }
}
