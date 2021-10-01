use libmicrodot::graph::Graph;
use libmicrodot::graphviz::GraphVizExporter;
use libmicrodot::json::JsonExporter;
use libmicrodot::parser::parse_line;
use libmicrodot::{Command, Line};
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let mut graph = Graph::new();

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
                        let mut exporter = GraphVizExporter::new();
                        let dot = exporter.export(&graph);
                        eprintln!("// START.DOT //\n");
                        eprintln!("{}", dot);
                        eprintln!("// END.DOT //\n");
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
}
