use clap::{AppSettings, Clap, ValueHint};
use libmicrodot::graph::Graph;
use libmicrodot::helper::{GetNodeLabel, MicrodotHelper};
use libmicrodot::json::{empty_json_graph, JsonImporter};
use libmicrodot::repl::repl;
use libmicrodot::{Id, Label};
use rustyline::{Config, Editor};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Clap)]
#[clap(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
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

struct GraphGetNodeLabel {}

impl GetNodeLabel for GraphGetNodeLabel {
    fn get_node_label(&self, id: &Id) -> Option<Label> {
        //Some(Label::new("I dunno".to_string()))
        None
    }
}

fn main() -> Result<(), anyhow::Error> {
    let opts = Opts::parse();
    let history = opts.history();
    let json_file = opts.file();

    let mut graph = load_graph(&json_file)?;
    let gnl = GraphGetNodeLabel {};

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

    repl(&mut rl, &json_file, &mut graph)?;

    rl.save_history(&history).unwrap();

    Ok(())
}

fn load_graph(json_file: &PathBuf) -> Result<Graph, anyhow::Error> {
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
    let mut graph = importer.import()?;
    Ok(graph)
}
