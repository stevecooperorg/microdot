use crate::graphviz::{DisplayMode, GraphVizExporter};
use crate::json::JsonExporter;
use crate::parser::parse_line;
use crate::{graphviz, svg, Command, Interaction};
use microdot_core::graph::Graph;
use microdot_core::{CommandResult, Line};
use rustyline::error::ReadlineError;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub fn repl<I: Interaction>(
    interaction: &mut I,
    json_file: &Path,
    graph: Arc<RwLock<Graph>>,
) -> Result<(), anyhow::Error> {
    loop {
        let readline = interaction.read(">> ");

        // when we start, make sure the existing pic is up to date.
        {
            let graph = graph.write().unwrap();
            let interactive_dot_file = save_file(json_file, &graph)?;
            if interaction.should_compile_dot() {
                compile_dot(interactive_dot_file);
            }
        }

        match readline {
            Ok(line) => {
                interaction.add_history(line.as_str());

                let line = Line::new(line);

                let command = parse_line(line);

                match command {
                    Command::GraphCommand(graph_command) => {
                        let mut graph = graph.write().unwrap();
                        interaction.log(format!("({})", graph.apply_command(graph_command)));
                        let interactive_dot_file = save_file(json_file, &graph)?;
                        if interaction.should_compile_dot() {
                            compile_dot(interactive_dot_file);
                        }
                    }
                    Command::ShowHelp => interaction.log(include_str!("help.txt")),
                    Command::RenameNodeUnlabelled { .. } => {
                        // no need to act, this is for auto-complete
                    }
                    Command::Show => {
                        let svg_file = json_file.with_extension("svg");
                        let svg_file = std::fs::canonicalize(svg_file)
                            .expect("could not canconcicalise file path");
                        let result = svg::open_in_gapplin(&svg_file);
                        interaction.log(result.to_string());
                    }
                    Command::PrintDot => {
                        let graph = graph.read().unwrap();
                        let mut exporter = GraphVizExporter::new(DisplayMode::Interactive);
                        let out = exporter.export(&graph);
                        interaction.log(out);
                        interaction.log("Dot printed");
                    }
                    Command::PrintJson => {
                        let graph = graph.read().unwrap();
                        let mut exporter = JsonExporter::new();
                        let out = exporter.export(&graph);
                        interaction.log(out);
                        interaction.log("Json printed");
                    }
                    Command::Search { sub_label } => {
                        let mut graph = graph.write().unwrap();
                        interaction.log(format!("({})", graph.highlight_search_results(sub_label)));

                        // save the file so we get color highlights.
                        let interactive_dot_file = save_file(json_file, &graph)?;
                        if interaction.should_compile_dot() {
                            compile_dot(interactive_dot_file);
                        }
                    }
                    Command::Save => {
                        let graph = graph.read().unwrap();
                        let interactive_dot_file = save_file(json_file, &graph)?;
                        interaction.log(format!(
                            "Saved json: {}, interactive dot: {}",
                            json_file.to_string_lossy(),
                            interactive_dot_file.to_string_lossy()
                        ));
                        compile_dot(interactive_dot_file);
                    }
                    Command::ParseError { .. } => {
                        interaction.log("could not understand command; try 'h' for help")
                    }
                    Command::Exit => break,
                }
            }
            Err(ReadlineError::Interrupted) => {
                interaction.log("CTRL-C");

                break;
            }
            Err(ReadlineError::Eof) => {
                interaction.log("CTRL-D");

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

fn save_file(json_file: &Path, graph: &Graph) -> Result<PathBuf, anyhow::Error> {
    let mut json_exporter = JsonExporter::new();
    let json = json_exporter.export(graph);
    std::fs::write(json_file, json)?;

    let mut dot_exporter = GraphVizExporter::new(DisplayMode::Interactive);
    let interactive_dot = dot_exporter.export(graph);
    let interactive_dot_file = json_file.with_extension("dot");
    std::fs::write(&interactive_dot_file, interactive_dot)?;

    Ok(interactive_dot_file)
}

fn compile_dot(interactive_dot_file: PathBuf) -> CommandResult {
    let msg = match graphviz::compile_dot(&interactive_dot_file, DisplayMode::Interactive) {
        Ok(_) => format!(
            "compiled interactive dot: {}",
            interactive_dot_file.to_string_lossy()
        ),
        Err(e) => format!("failed to compile interactive dot: {}", e),
    };

    CommandResult::new(format!("{}", msg))
}
