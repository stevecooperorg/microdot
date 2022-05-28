use clap::{Parser, ValueHint};
use libmicrodot::helper::{GetNodeLabel, MicrodotHelper};
use libmicrodot::json::{empty_json_graph, JsonImporter};
use libmicrodot::repl::repl;
use microdot_core::graph::*;
use microdot_core::*;
use rustyline::{Config, Editor};
use std::fs::File;
use std::io::Read;
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
    let json_content = if json_file.exists() {
        println!(
            "loading existing graph from {}",
            json_file.to_string_lossy()
        );
        let mut f = File::open(&json_file)?;
        let mut s = "".to_string();
        f.read_to_string(&mut s)?;
        s
    } else {
        empty_json_graph()
    };

    let importer = JsonImporter::new(json_content);
    let graph = importer.import()?;
    Ok(graph)
}
