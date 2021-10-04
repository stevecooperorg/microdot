use clap::{AppSettings, Clap, ValueHint};
use libmicrodot::graph::Graph;
use libmicrodot::graphviz::GraphVizExporter;
use libmicrodot::json::{empty_json_graph, JsonExporter, JsonImporter};
use libmicrodot::parser::parse_line;
use libmicrodot::{Command, CommandResult, graphviz, Line};
use rustyline::error::ReadlineError;
use rustyline::Editor;
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

fn main() -> Result<(), anyhow::Error> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();

    let Opts {
        history,
        file: json_file,
    } = Opts::parse();

    let history = history.unwrap_or_else(|| dirs::home_dir().unwrap().join(".microdot_history"));
    let json_file = json_file.unwrap_or_else(|| dirs::home_dir().unwrap().join("microdot_graph.json"));

    if rl.load_history(&history).is_err() {
        println!("No previous history.");
    }

    let json_content = if json_file.exists() {
        println!("loading existing graph from {}", json_file.to_string_lossy());
        let mut f = File::open(&json_file)?;
        let mut s = "".to_string();
        f.read_to_string(&mut s)?;
        s
    } else {
        empty_json_graph()
    };

    let importer = JsonImporter::new(json_content);
    let mut graph = importer.import()?;

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                let line = Line::new(line);

                let command = parse_line(line);

                match command {
                    Command::GraphCommand(graph_command) => {
                        println!("({})", graph.apply_command(graph_command));
                        let dot_file = save_file(&json_file, &graph)?;
                        compile_dot(dot_file);
                    }
                    Command::ShowHelp => println!(include_str!("help.txt")),
                    Command::PrintDot => {
                        let mut exporter = GraphVizExporter::new();
                        let out = exporter.export(&graph);
                        println!("{}", out);
                        println!("Dot printed");
                    }
                    Command::PrintJson => {
                        let mut exporter = JsonExporter::new();
                        let out = exporter.export(&graph);
                        println!("{}", out);
                        println!("Json printed");
                    }
                    Command::Save => {
                        let dot_file = save_file(&json_file, &graph)?;
                        compile_dot(dot_file);
                    }
                    Command::ParseError { .. } => {
                        println!("could not understand command; try 'h' for help")
                    }
                    Command::Exit => break,
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");

                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");

                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);

                break;
            }
        }
    }

    rl.save_history("history.txt").unwrap();

    Ok(())
}

fn save_file(json_file: &PathBuf, graph: &Graph) -> Result<PathBuf, anyhow::Error> {
    let mut json_exporter = JsonExporter::new();
    let json = json_exporter.export(&graph);
    let mut dot_exporter = GraphVizExporter::new();
    let dot = dot_exporter.export(&graph);
    let dot_file = json_file.with_extension("dot");
    std::fs::write(&json_file, json)?;
    std::fs::write(&dot_file, dot)?;
    println!(
        "Saved json: {}, dot: {}",
        json_file.to_string_lossy(),
        dot_file.to_string_lossy()
    );
    Ok(dot_file)
}

fn compile_dot(dot_file: PathBuf) -> CommandResult {
    match graphviz::compile_dot(&dot_file) {
        Ok(_) => CommandResult::new(format!("compiled dot: {}", dot_file.to_string_lossy())),
        Err(e) => CommandResult::new(format!("failed to compile dot: {}", e.to_string()))
    }
}
