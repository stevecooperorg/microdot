use clap::{Parser, ValueHint};
use libmicrodot::helper::{GetNodeLabel, MicrodotHelper};
use libmicrodot::json::JsonImporter;
use libmicrodot::repl::repl;
use libmicrodot::web::run_web_server;
use microdot_core::graph::*;
use microdot_core::*;
use rustyline::{Config, Editor};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

/// a REPL and terminal ui for dot and graphviz
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    /// Sets the data source file, defaults to ~/.microdot_graph.json
    #[clap(short, long,  value_hint = ValueHint::FilePath)]
    file: Option<PathBuf>,

    /// Sets a custom history file location, defaults to ~/.microdot_history
    #[clap(long, value_hint = ValueHint::FilePath)]
    history: Option<PathBuf>,

    /// Optional port number for the web server
    #[clap(long)]
    port: Option<u16>,
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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    eprintln!("Microdot: a REPL and terminal ui for dot and graphviz.");
    let opts = Opts::parse();
    let history = opts.history();
    let json_file = opts.file();
    let json_file = if !json_file.exists() && json_file.extension().is_none() {
        json_file.with_extension("json")
    } else {
        json_file.to_path_buf()
    };

    let graph = load_graph_if_exists(&json_file)?;
    let graph = Arc::new(RwLock::new(graph));
    let gnl = GraphGetNodeLabel {
        graph: graph.clone(),
    };

    let h = MicrodotHelper::new(&gnl);
    let config = Config::default();
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(h));

    // Create reload channel
    let (reload_tx, reload_rx) = mpsc::unbounded_channel();

    if rl.load_history(&history).is_err() {
        println!("No previous history at {}.", history.to_string_lossy());
    } else {
        println!(
            "Loaded previous history from {}.",
            history.to_string_lossy()
        );
    }

    if let Some(port) = opts.port {
        let svg_path = json_file.with_extension("svg");
        let html_path = json_file.with_extension("html");
        let json_path = json_file.clone();
        let reload_rx = reload_rx;
        tokio::spawn(async move {
            if let Err(e) = run_web_server(port, svg_path, html_path, json_path, reload_rx).await {
                eprintln!("Failed to start web server: {}", e);
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    repl(&mut rl, &json_file, graph, reload_tx)?;

    rl.save_history(&history).unwrap();

    Ok(())
}

fn load_graph_if_exists(json_file: &Path) -> Result<Graph, anyhow::Error> {
    if !json_file.exists() {
        return Ok(Graph::new());
    }

    JsonImporter::load(json_file)
}
