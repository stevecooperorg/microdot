use clap::{AppSettings, Clap, ValueHint};
use libmicrodot::graph::Graph;
use libmicrodot::graphviz::GraphVizExporter;
use libmicrodot::json::{empty_json_graph, JsonExporter, JsonImporter};
use libmicrodot::parser::parse_line;
use libmicrodot::{graphviz, Command, CommandResult, Line, Interaction};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use libmicrodot::repl::repl;

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

fn main() -> Result<(), anyhow::Error> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();

    let Opts {
        history,
        file: json_file,
    } = Opts::parse();

    let history = history.unwrap_or_else(|| dirs::home_dir().unwrap().join(".microdot_history"));
    let json_file =
        json_file.unwrap_or_else(|| dirs::home_dir().unwrap().join("microdot_graph.json"));

    if rl.load_history(&history).is_err() {
        println!("No previous history at {}.", history.to_string_lossy());
    } else {
        println!(
            "Loaded previous history from {}.",
            history.to_string_lossy()
        );
    }

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

    repl(&mut rl, &json_file, &mut graph)?;

    rl.save_history(&history).unwrap();

    Ok(())
}

