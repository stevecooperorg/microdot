use crate::graph::Graph;
use crate::graphviz::{DisplayMode, GraphVizExporter};
use crate::json::JsonExporter;
use crate::parser::parse_line;
use crate::{graphviz, Command, CommandResult, Interaction, Line};
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

        match readline {
            Ok(line) => {
                interaction.add_history(line.as_str());

                let line = Line::new(line);

                let command = parse_line(line);

                match command {
                    Command::GraphCommand(graph_command) => {
                        let mut graph = graph.write().unwrap();
                        interaction.log(format!("({})", graph.apply_command(graph_command)));
                        let (interactive_dot_file, presentation_dot_file) =
                            save_file(json_file, &graph)?;
                        if interaction.should_compile_dot() {
                            compile_dot(interactive_dot_file, presentation_dot_file);
                        }
                    }
                    Command::ShowHelp => interaction.log(include_str!("help.txt")),
                    Command::Show => {
                        let svg_file = json_file.with_extension("svg");
                        let result = graphviz::open_in_gapplin(&svg_file);
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
                        let (interactive_dot_file, presentation_dot_file) =
                            save_file(json_file, &graph)?;
                        if interaction.should_compile_dot() {
                            compile_dot(interactive_dot_file, presentation_dot_file);
                        }
                    }
                    Command::Save => {
                        let graph = graph.read().unwrap();
                        let (interactive_dot_file, presentation_dot_file) =
                            save_file(json_file, &graph)?;
                        interaction.log(format!(
                            "Saved json: {}, interactive dot: {}, presentation dot: {}",
                            json_file.to_string_lossy(),
                            interactive_dot_file.to_string_lossy(),
                            presentation_dot_file.to_string_lossy()
                        ));
                        compile_dot(interactive_dot_file, presentation_dot_file);
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

fn save_file(json_file: &Path, graph: &Graph) -> Result<(PathBuf, PathBuf), anyhow::Error> {
    let mut json_exporter = JsonExporter::new();
    let json = json_exporter.export(graph);
    std::fs::write(&json_file, json)?;

    let mut dot_exporter = GraphVizExporter::new(DisplayMode::Interactive);
    let interactive_dot = dot_exporter.export(graph);
    let interactive_dot_file = json_file.with_extension("dot");
    std::fs::write(&interactive_dot_file, interactive_dot)?;

    let mut dot_exporter = GraphVizExporter::new(DisplayMode::Presentation);
    let presentation_dot = dot_exporter.export(graph);
    let presentation_dot_file = json_file.with_extension("presentation.dot");
    std::fs::write(&presentation_dot_file, presentation_dot)?;

    Ok((interactive_dot_file, presentation_dot_file))
}

fn compile_dot(interactive_dot_file: PathBuf, presentation_dot_file: PathBuf) -> CommandResult {
    let msg1 = match graphviz::compile_dot(&interactive_dot_file, DisplayMode::Interactive) {
        Ok(_) => format!(
            "compiled interactive dot: {}",
            interactive_dot_file.to_string_lossy()
        ),
        Err(e) => format!("failed to compile interactive dot: {}", e.to_string()),
    };

    let msg2 = match graphviz::compile_dot(&presentation_dot_file, DisplayMode::Presentation) {
        Ok(_) => format!(
            "compiled presentation dot: {}",
            presentation_dot_file.to_string_lossy()
        ),
        Err(e) => format!("failed to compile presentation dot: {}", e.to_string()),
    };

    CommandResult::new(format!("{}\n{}", msg1, msg2))
}
