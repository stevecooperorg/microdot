use clap::{AppSettings, Clap, ValueHint};
use crate::graph::Graph;
use crate::graphviz::GraphVizExporter;
use crate::json::{empty_json_graph, JsonExporter, JsonImporter};
use crate::parser::parse_line;
use crate::{graphviz, Command, CommandResult, Line, Interaction};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn repl<I: Interaction>(interaction: &mut I, json_file: &PathBuf, graph: &mut Graph) -> Result<(), anyhow::Error> {
    loop {
        let readline = interaction.read(">> ");

        match readline {
            Ok(line) => {
                interaction.add_history(line.as_str());

                let line = Line::new(line);

                let command = parse_line(line);

                match command {
                    Command::GraphCommand(graph_command) => {
                        interaction.log(format!("({})", graph.apply_command(graph_command)));
                        let dot_file = save_file(&json_file, &graph)?;
                        compile_dot(dot_file);
                    }
                    Command::ShowHelp => interaction.log(format!(include_str!("help.txt"))),
                    Command::PrintDot => {
                        let mut exporter = GraphVizExporter::new();
                        let out = exporter.export(&graph);
                        interaction.log(format!("{}", out));
                        interaction.log(format!("Dot printed"));
                    }
                    Command::PrintJson => {
                        let mut exporter = JsonExporter::new();
                        let out = exporter.export(&graph);
                        interaction.log(format!("{}", out));
                        interaction.log(format!("Json printed"));
                    }
                    Command::Search { sub_label } => {
                        interaction.log(format!("({})", graph.search(sub_label)));

                        // save the file so we get color highlights.
                        let dot_file = save_file(&json_file, &graph)?;
                        compile_dot(dot_file);
                    }
                    Command::Save => {
                        let dot_file = save_file(&json_file, &graph)?;
                        interaction.log(format!(
                            "Saved json: {}, dot: {}",
                            json_file.to_string_lossy(),
                            dot_file.to_string_lossy()
                        ));
                        compile_dot(dot_file);
                    }
                    Command::ParseError { .. } => {
                        interaction.log(format!("could not understand command; try 'h' for help"))
                    }
                    Command::Exit => break,
                }
            }
            Err(ReadlineError::Interrupted) => {
                interaction.log(format!("CTRL-C"));

                break;
            }
            Err(ReadlineError::Eof) => {
                interaction.log(format!("CTRL-D"));

                break;
            }
            Err(err) => {
                interaction.log(format!("Error: {:?}", err));

                break;
            }
        }
    }

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
    Ok(dot_file)
}

fn compile_dot(dot_file: PathBuf) -> CommandResult {
    match graphviz::compile_dot(&dot_file) {
        Ok(_) => CommandResult::new(format!("compiled dot: {}", dot_file.to_string_lossy())),
        Err(e) => CommandResult::new(format!("failed to compile dot: {}", e.to_string())),
    }
}
