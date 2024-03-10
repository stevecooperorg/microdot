use clap::{Parser, ValueHint};
use libmicrodot::helper::{GetNodeLabel, MicrodotHelper};
use libmicrodot::json::JsonImporter;
use libmicrodot::repl::repl;
use microdot_core::graph::*;
use microdot_core::*;
use rustyline::{Config, Editor};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long,  value_hint = ValueHint::FilePath)]
    file: Option<PathBuf>,

    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, value_hint = ValueHint::FilePath)]
    history: Option<PathBuf>,
}

impl Opts {
    fn history(&self) -> PathBuf {
        self.history
            .clone()
            .unwrap_or_else(|| dirs::home_dir().unwrap().join(".microdot_history"))
    }

    fn file(&self) -> PathBuf {
        self.file
            .clone()
            .unwrap_or_else(|| dirs::home_dir().unwrap().join("microdot_graph.json"))
    }
}

struct GraphGetNodeLabel {
    graph: Arc<RwLock<Graph>>,
}

impl GetNodeLabel for GraphGetNodeLabel {
    fn get_node_label(&self, id: &Id) -> Option<Label> {
        let graph = self.graph.read().unwrap();
        graph.find_node_label(id)
    }
}

fn main() -> Result<(), anyhow::Error> {
    let opts = Opts::parse();
    let history = opts.history();
    let json_file = opts.file();
    let json_file = if !json_file.exists() && json_file.extension().is_none() {
        json_file.with_extension("json")
    } else {
        json_file.to_path_buf()
    };

    let graph = load_graph(&json_file)?;
    let graph = Arc::new(RwLock::new(graph));
    let gnl = GraphGetNodeLabel {
        graph: graph.clone(),
    };

    let h = MicrodotHelper::new(&gnl);
    let config = Config::default();
    let mut rl = Editor::with_config(config);
    rl.set_helper(Some(h));

    if rl.load_history(&history).is_err() {
        println!("No previous history at {}.", history.to_string_lossy());
    } else {
        println!(
            "Loaded previous history from {}.",
            history.to_string_lossy()
        );
    }

    repl(&mut rl, &json_file, graph)?;

    rl.save_history(&history).unwrap();

    Ok(())
}

fn load_graph(json_file: &Path) -> Result<Graph, anyhow::Error> {
    if !json_file.exists() {
        return Ok(Graph::new());
    }

    JsonImporter::load(json_file)
}
