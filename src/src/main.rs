use rustyline::error::ReadlineError;
use rustyline::Editor;
use libmicrodot::graph::Graph;
use libmicrodot::{Command, Line};
use libmicrodot::graphviz::Exporter;
use libmicrodot::parser::parse_line;

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
                    Command::GraphCommand(graph_command) => println!("{}", graph.apply_command(graph_command)),
                    Command::ShowHelp => println!(include_str!("help.txt")),
                    Command::PrintGraph => {
                        let mut exporter = Exporter::new();
                        let dot = exporter.export(&graph);
                        println!("{}", dot);
                    },
                    Command::ParseError { .. } => println!("could not understand command; try 'h' for help"),
                    Command::Exit => break
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}