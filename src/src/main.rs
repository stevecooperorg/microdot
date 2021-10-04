use clap::{AppSettings, Clap, ValueHint};
use libmicrodot::graph::Graph;
use libmicrodot::graphviz::GraphVizExporter;
use libmicrodot::json::{JsonExporter, JsonImporter};
use libmicrodot::parser::parse_line;
use libmicrodot::{Command, Line};
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
    #[clap(short, long, default_value = "~/microdot_graph.json", value_hint = ValueHint::FilePath)]
    file: PathBuf,

    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "~/.microdot_history", value_hint = ValueHint::FilePath)]
    history: PathBuf,
}

fn main() -> Result<(), anyhow::Error> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();

    let Opts {
        history,
        file: json_file,
    } = Opts::parse();

    if rl.load_history(&history).is_err() {
        println!("No previous history.");
    }

    let json_content = if json_file.exists() {
        let mut f = File::open(&json_file)?;
        let mut s = "".to_string();
        f.read_to_string(&mut s)?;
        s
    } else {
        r#"{ "nodes":[], "edges":[] }"#.to_string()
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
                        save_file(&json_file, &graph)?;
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
                        save_file(&json_file, &graph)?;
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

fn save_file(json_file: &PathBuf, graph: &Graph) -> Result<(), anyhow::Error> {
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
    Ok(())
}
