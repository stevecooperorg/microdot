use crate::graphviz::{compile_dot, DisplayMode, GraphVizExporter, OutputFormat};
use crate::repl::repl;
use crate::Interaction;
use anyhow::{anyhow, Result};
use microdot_core::graph::Graph;
use std::collections::VecDeque;
use std::path::*;
use std::sync::{Arc, RwLock};
use unfold::Unfold;

pub fn git_root() -> Result<PathBuf> {
    let current_dir: &Path = &std::env::current_dir()?;
    let mut ancestors = Unfold::new(|path| path.parent().unwrap(), current_dir);

    fn has_git_dir(path: &Path) -> bool {
        std::fs::read_dir(path)
            .unwrap()
            .flatten()
            .any(|p| p.file_name() == ".git")
    }

    let git_root = ancestors
        .find(|&path| has_git_dir(path))
        .ok_or_else(|| anyhow!("could not find the git root"))?;

    Ok(git_root.into())
}

pub fn compile_input_string_content(text_file: PathBuf) -> PathBuf {
    assert!(
        text_file.exists(),
        "text file does not exist: '{}'",
        text_file.to_string_lossy()
    );

    // read the file as lines and run it through the repl;
    let graph = Arc::new(RwLock::new(Graph::new()));

    let text_content = std::fs::read_to_string(&text_file).expect("could not read file");
    let lines: VecDeque<_> = text_content.lines().map(|l| l.to_string()).collect();
    let mut auto_interaction = AutoInteraction::new(lines);
    repl(
        &mut auto_interaction,
        &text_file.with_extension("json"),
        graph.clone(),
    )
    .expect("error in repl");

    let mut exporter = GraphVizExporter::new(DisplayMode::Interactive);
    let graph = graph.read().unwrap();
    let exported = exporter.export(&graph);

    let dot_file = text_file.with_extension("dot");
    write_if_different(&dot_file, exported).expect("could not write dot file");

    let log_file = text_file.with_extension("log");
    write_if_different(&log_file, auto_interaction.log()).expect("could not write log file");

    compile_dot(&dot_file, DisplayMode::Interactive, OutputFormat::Svg)
        .unwrap_or_else(|_| panic!("Could not compile '{}'", dot_file.to_string_lossy()));

    log_file
}

pub fn write_if_different<P: AsRef<Path>, C: AsRef<[u8]>>(
    path: P,
    contents: C,
) -> std::io::Result<()> {
    let path = path.as_ref();
    let contents = contents.as_ref();

    let needs_write = match std::fs::read(path) {
        Ok(current_content) => current_content == contents,
        Err(_) => true,
    };

    if needs_write {
        std::fs::write(path, contents)
    } else {
        Ok(())
    }
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
    fn read(&mut self, _prompt: &str) -> rustyline::Result<String> {
        match self.lines.pop_front() {
            Some(line) => Ok(line),
            None => Err(rustyline::error::ReadlineError::Eof),
        }
    }

    fn add_history<S: AsRef<str> + Into<String>>(&mut self, history: S) -> bool {
        self.log.push_str(">> ");
        self.log.push_str(&history.into());
        self.log.push('\n');
        true
    }

    fn log<S: AsRef<str> + Into<String>>(&mut self, message: S) {
        self.log.push_str(&message.into());
        self.log.push('\n');
    }

    fn should_compile_dot(&self) -> bool {
        false
    }
}
