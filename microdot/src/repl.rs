use crate::graphviz::{DisplayMode, GraphVizExporter};
use crate::json::JsonExporter;
use crate::parser::parse_line;
use crate::util::write_if_different;
use crate::{graphviz, svg, Command, Interaction};
use anyhow::{anyhow, Result};
use microdot_core::graph::Graph;
use microdot_core::pet::{find_cost, find_longest_path, CostCalculator};
use microdot_core::{CommandResult, Line};
use rustyline::error::ReadlineError;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::UnboundedSender;

pub fn repl<I: Interaction>(
    interaction: &mut I,
    json_file: &Path,
    graph: Arc<RwLock<Graph>>,
    reload_tx: UnboundedSender<()>,
) -> Result<()> {
    loop {
        let readline = interaction.read(">> ");

        // when we start, make sure the existing pic is up to date.
        compile_graph(interaction, json_file, &graph, Some(&reload_tx))?;

        let dirty = match readline {
            Ok(line) => {
                interaction.add_history(line.as_str());

                let line = Line::new(line);

                let command = parse_line(line);

                match command {
                    Command::GraphCommand(graph_command) => {
                        let mut graph = graph.write().unwrap();
                        let applied = graph.apply_command(graph_command);
                        interaction.log(format!("({})", applied));
                        true
                    }
                    Command::ShowHelp => {
                        interaction.log(include_str!("help.txt"));
                        false
                    }
                    Command::RenameNodeUnlabelled { .. } => {
                        // no need to act, this is for auto-complete
                        false
                    }
                    Command::Show => {
                        let svg_file = json_file.with_extension("svg");
                        let svg_file = std::fs::canonicalize(svg_file)
                            .expect("could not canconcicalise file path");
                        let result = svg::open_in_gapplin(&svg_file);
                        interaction.log(result.to_string());
                        false
                    }
                    Command::PrintDot => {
                        let graph = graph.read().unwrap();
                        let mut exporter = GraphVizExporter::new(DisplayMode::Interactive);
                        let out = exporter.export_dot(&graph);
                        interaction.log(out);
                        interaction.log("Dot printed");
                        false
                    }
                    Command::PrintJson => {
                        let graph = graph.read().unwrap();
                        let mut exporter = JsonExporter::new();
                        let out = exporter.export_json(&graph);
                        interaction.log(out);
                        interaction.log("Json printed");
                        false
                    }
                    Command::Search { sub_label } => {
                        let mut graph = graph.write().unwrap();
                        interaction.log(format!("({})", graph.highlight_search_results(sub_label)));
                        true
                    }
                    Command::Save => {
                        interaction.log(format!("saving to {}", json_file.to_string_lossy()));
                        true
                    }
                    Command::CriticalPathAnalysis { variable_name } => {
                        let graph = graph.read().unwrap();
                        interaction.log(format!(
                            "performing critical path analysis using variable {}",
                            variable_name
                        ));

                        let longest_path =
                            find_longest_path(&graph, CostCalculator::new(variable_name.clone()));
                        for (i, node) in longest_path.ids.iter().enumerate() {
                            if let Some(label) = graph.find_node_label(node) {
                                let val = match graph.find_node_variable_value(node, &variable_name)
                                {
                                    Some(val) => format!("{}", val),
                                    None => "".to_string(),
                                };
                                interaction.log(format!("Step {}: {}: {}", i + 1, val, label));
                            }
                        }

                        if !longest_path.ids.is_empty() {
                            interaction.log("====================");
                            if let Some(cost) = longest_path.cost {
                                interaction.log(format!("Total cost: {}", cost));
                            }
                            interaction.log(format!("Total length: {}", longest_path.ids.len()));
                        }

                        false
                    }
                    Command::ParseError { .. } => {
                        interaction.log("could not understand command; try 'h' for help");
                        false
                    }
                    Command::Exit => return Ok(()),
                    Command::CostAnalysis { variable_name } => {
                        let graph = graph.read().unwrap();
                        interaction.log(format!(
                            "performing cost analysis using variable {}",
                            variable_name
                        ));
                        let cost = find_cost(&graph, CostCalculator::new(variable_name.clone()));
                        interaction.log(format!("Total cost: {}", cost));
                        false
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                interaction.log("CTRL-C");

                return Ok(());
            }
            Err(ReadlineError::Eof) => {
                interaction.log("CTRL-D");

                return Ok(());
            }
            Err(err) => {
                interaction.log(format!("Error: {:?}", err));
                return Err(anyhow::anyhow!("readline error: {}", err.to_string()));
            }
        };

        if dirty {
            compile_graph(interaction, json_file, &graph, Some(&reload_tx))?;
        }
    }
}

enum RenderMethod {
    GraphViz,
}

const RENDER_METHOD: RenderMethod = RenderMethod::GraphViz;

fn compile_graph<I: Interaction>(
    interaction: &mut I,
    json_file: &Path,
    graph: &Arc<RwLock<Graph>>,
    reload_tx: Option<&UnboundedSender<()>>,
) -> Result<()> {
    let graph = match graph.write() {
        Ok(graph) => graph,
        Err(e) => return Err(anyhow!(e.to_string())),
    };
    match RENDER_METHOD {
        RenderMethod::GraphViz => {
            // causes problems in unit tests, because interim results have the file saving partial
            // results, which tells cargo watch that it should recompile
            let interactive_dot_file = save_dot_file(json_file, &graph)?;
            if interaction.should_compile() {
                compile_dot(interactive_dot_file, reload_tx);
            }
        }
    }

    Ok(())
}

fn save_dot_file(json_file: &Path, graph: &Graph) -> Result<PathBuf> {
    let mut json_exporter = JsonExporter::new();
    let json = json_exporter.export_json(graph);
    write_if_different(json_file, json)?;

    let mut dot_exporter = GraphVizExporter::new(DisplayMode::Interactive);
    let interactive_dot = dot_exporter.export_dot(graph);
    let interactive_dot_file = json_file.with_extension("dot");
    write_if_different(&interactive_dot_file, interactive_dot)?;

    Ok(interactive_dot_file)
}

fn compile_dot(interactive_dot_file: PathBuf, reload_tx: Option<&UnboundedSender<()>>) -> CommandResult {
    let svg_compile = graphviz::compile(&interactive_dot_file);

    if let Ok(_) = svg_compile {
        if let Some(tx) = reload_tx {
            let _ = tx.send(());
        }
    }

    let msg = match svg_compile {
        Ok(_) => format!(
            "compiled interactive dot: {}",
            interactive_dot_file.to_string_lossy()
        ),
        Err(e) => format!("failed to compile interactive dot to svg: {}", e),
    };

    CommandResult::new(msg)
}
