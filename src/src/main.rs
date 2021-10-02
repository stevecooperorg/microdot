use libmicrodot::graph::Graph;
use libmicrodot::graphviz::GraphVizExporter;
use libmicrodot::json::{JsonExporter, JsonImporter};
use libmicrodot::parser::parse_line;
use libmicrodot::{Command, Line};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() -> Result<(), anyhow::Error> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    // TODO: grab this from args?
    let json_file = Path::new("graph.json");

    let json_content = if json_file.exists() {
        let mut f = File::open(json_file)?;
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

fn save_file(json_file: &&Path, graph: &Graph) -> Result<(), anyhow::Error> {
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
