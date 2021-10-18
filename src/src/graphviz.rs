use crate::colors::Color;
use crate::graph::Graph;
use crate::palettes::PaletteReader;
use crate::{CommandResult, Exporter, Id, Label, NodeHighlight};
use command_macros::cmd;
use hyphenation::{Language, Load, Standard};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use textwrap::word_separators::UnicodeBreakProperties;
use textwrap::wrap_algorithms::OptimalFit;
use textwrap::{fill, Options};

const PALETTE_NAME: &str = "antarctica_evening";
const GAPPLIN_PATH: &str = "/Applications/Gapplin.app/Contents/MacOS/Gapplin";

struct ColorScheme {
    font_color: String,
    fill_color: String,
    stroke_color: String,
    node_border_width: f64,
}

impl ColorScheme {
    pub fn from_entry(i: usize) -> Self {
        let content = include_str!("./palettes.txt");
        let reader = PaletteReader {};
        let palettes = reader.read(content).expect("couldn't read palette");
        let palette = palettes.get(PALETTE_NAME).unwrap();

        let stroke_color = Color::black();
        let fill_color = palette.get_color(i);
        let font_color = stroke_color;
        Self {
            font_color: font_color.to_html_string(),
            fill_color: fill_color.to_html_string(),
            stroke_color: stroke_color.to_html_string(),
            node_border_width: 3.0f64,
        }
    }

    fn normal() -> Self {
        ColorScheme::from_entry(0)
    }

    fn search_result() -> Self {
        ColorScheme::from_entry(1)
    }

    fn current() -> Self {
        ColorScheme::from_entry(2)
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

pub fn open_in_gapplin(svg_path: &Path) -> CommandResult {
    let viewer = GAPPLIN_PATH;
    let svg_path = &svg_path.to_string_lossy().to_string();
    if Path::new(viewer).exists() {
        let mut cmd = std::process::Command::new(viewer);
        cmd.arg(svg_path);
        match cmd.spawn() {
            Ok(_) => CommandResult(format!("Opened {} in {}", svg_path, viewer)),
            Err(e) => CommandResult(format!(
                "Could not open {} in {}: {}",
                svg_path,
                viewer,
                e.to_string()
            )),
        }
    } else {
        CommandResult(format!("Could not open {} in {}", svg_path, viewer))
    }
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

fn hashtag_signature(input: &str) -> usize {
    let rx = Regex::new("#[A-Z][A-Z0-9]*").expect("not a regex");
    let mut hashes = HashSet::new();
    for hash in rx.captures_iter(input) {
        let hash = hash.get(0).unwrap().as_str();
        hashes.insert(hash);
    }

    if hashes.is_empty() {
        return 0;
    }

    let mut hashes: Vec<_> = hashes.into_iter().collect();
    hashes.sort();

    let combo: String = hashes.join("");
    let combo = combo.as_bytes();
    let digest = md5::compute(combo);
    let digest_u8: [u8; 16] = digest.into();
    let mut hash = 0usize;
    for byte in digest_u8 {
        hash += byte as usize;
    }
    hash
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

    let out = path.with_extension("svg");

    cmd!(dot(path)("-Tsvg")("-o")(out)).output()?;

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

        let wrapping_options: Options<OptimalFit, UnicodeBreakProperties, Standard> = {
            let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
            Options::new(40).word_splitter(dictionary)
        };

        let hash = hashtag_signature(&label.0);

        let color_scheme = ColorScheme::from_entry(hash);

        let ColorScheme {
            font_color,
            fill_color,
            stroke_color,
            node_border_width,
            ..
        } = color_scheme;

        let label_text = match self.display_mode {
            DisplayMode::Interactive => {
                let unwrapped = format!("{}: {}", id.0, label.0);
                fill(&unwrapped, &wrapping_options)
            }
            DisplayMode::Presentation => label.0.clone(),
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
        self.inner_content.push(';');
        self.inner_content.push('\n');
    }

    fn add_edge(&mut self, id: &Id, from: &Id, to: &Id) {
        if self.is_first_edge {
            self.inner_content.push('\n');
            self.is_first_edge = false;
        }

        let edge_params = hashmap! {
            "id" => id.0.clone(),
            "escaped_id" => escape_id(&id.0),
            "escaped_from" => escape_id(&from.0),
            "escaped_to" => escape_id(&to.0),
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
        let template = include_str!("template.dot");

        graph.export(self);

        let rankdir = if self.is_left_right { "LR" } else { "TB" };

        let edge_color = ColorScheme::normal().stroke_color;

        template
            .replace("${RANKDIR}", rankdir)
            .replace("${EDGECOLOR}", &edge_color)
            .replace("${INNER_CONTENT}", &self.inner_content)
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
    use crate::repl::repl;
    use crate::{GraphCommand, Id, Interaction, Label};
    use pom::set::Set;
    use std::collections::{HashSet, VecDeque};
    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};

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

        graph.highlight_search_results(Label::new("abc"));

        let mut exporter = GraphVizExporter::new(DisplayMode::Interactive);

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

        let compile_result = compile_dot(&dot_file, DisplayMode::Interactive);
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

        fn should_compile_dot(&self) -> bool {
            false
        }
    }

    fn compile_input_string_content(text_file: PathBuf) {
        assert!(
            text_file.exists(),
            "text file does not exist: '{}'",
            text_file.to_string_lossy()
        );

        // read the file as lines and run it through the repl;
        let mut graph = Arc::new(RwLock::new(Graph::new()));

        let text_content = std::fs::read_to_string(&text_file).expect("could not read file");
        let lines: VecDeque<_> = text_content.lines().map(|l| l.to_string()).collect();
        let mut auto_interaction = AutoInteraction::new(lines);
        repl(
            &mut auto_interaction,
            &text_file.with_extension("json"),
            graph.clone(),
        );

        let mut exporter = GraphVizExporter::new(DisplayMode::Interactive);
        let graph = graph.read().unwrap();
        let exported = exporter.export(&graph);

        let dot_file = text_file.with_extension("dot");
        std::fs::write(&dot_file, exported).expect("could not write dot file");

        let log_file = text_file.with_extension("log");
        std::fs::write(&log_file, auto_interaction.log()).expect("could not write log file");

        compile_dot(&dot_file, DisplayMode::Interactive).expect(&format!(
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

    #[test]
    fn extracts_hashtags_right() {
        fn eq(a: &str, b: &str) {
            let ai = hashtag_signature(a);
            let bi = hashtag_signature(b);
            assert_eq!(
                ai, bi,
                "signatures are not the same for '{}' and '{}'",
                a, b
            )
        }

        fn ne(a: &str, b: &str) {
            let ai = hashtag_signature(a);
            let bi = hashtag_signature(b);
            assert_ne!(ai, bi, "signatures are the same for '{}' and '{}'", a, b)
        }

        assert_eq!(0, hashtag_signature("no hashtags here"));
        assert_ne!(0, hashtag_signature("hashtag! #HASH"));
        eq("a #HASH", "b #HASH");
        eq("#HASH a", "a #HASH");
        eq("#A #B", "#B #A");
        eq("#A #A", "#A");
        ne("#HASHA a", "a #HASHB");
    }
}
