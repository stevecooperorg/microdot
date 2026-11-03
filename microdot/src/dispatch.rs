//! Shared command dispatch for the REPL and TUI.

use crate::graphviz::{DisplayMode, GraphVizExporter};
use crate::json::JsonExporter;
use crate::svg;
use crate::Command;
use microdot_core::graph::Graph;
use microdot_core::pet::{find_cost, find_longest_path, CostCalculator};
use std::path::Path;
use std::sync::{Arc, RwLock};

#[derive(Debug, Default)]
pub struct DispatchResult {
    pub dirty: bool,
    pub quit: bool,
    pub messages: Vec<String>,
}

impl DispatchResult {
    fn msg(mut self, m: impl Into<String>) -> Self {
        self.messages.push(m.into());
        self
    }

    fn dirty(mut self) -> Self {
        self.dirty = true;
        self
    }

    fn quit(mut self) -> Self {
        self.quit = true;
        self
    }
}

/// Apply a parsed command to the graph. Does not persist; callers check `dirty`.
pub fn dispatch_command(
    command: Command,
    graph: &Arc<RwLock<Graph>>,
    json_file: &Path,
) -> DispatchResult {
    match command {
        Command::GraphCommand(graph_command) => {
            let mut graph = graph.write().unwrap();
            let applied = graph.apply_command(graph_command);
            DispatchResult::default().msg(format!("({})", applied)).dirty()
        }
        Command::ShowHelp => DispatchResult::default().msg(include_str!("help.txt")),
        Command::RenameNodeUnlabelled { .. } => DispatchResult::default(),
        Command::Show => {
            let svg_file = json_file.with_extension("svg");
            let svg_file =
                std::fs::canonicalize(&svg_file).unwrap_or_else(|_| svg_file.to_path_buf());
            let result = svg::open_in_gapplin(&svg_file);
            DispatchResult::default().msg(result.to_string())
        }
        Command::PrintDot => {
            let graph = graph.read().unwrap();
            let mut exporter = GraphVizExporter::new(DisplayMode::Interactive);
            let out = exporter.export_dot(&graph);
            DispatchResult::default()
                .msg(out)
                .msg("Dot printed")
        }
        Command::PrintJson => {
            let graph = graph.read().unwrap();
            let mut exporter = JsonExporter::new();
            let out = exporter.export_json(&graph);
            DispatchResult::default()
                .msg(out)
                .msg("Json printed")
        }
        Command::Search { sub_label } => {
            let mut graph = graph.write().unwrap();
            let applied = graph.highlight_search_results(sub_label);
            DispatchResult::default()
                .msg(format!("({})", applied))
                .dirty()
        }
        Command::Save => DispatchResult::default()
            .msg(format!("saving to {}", json_file.to_string_lossy()))
            .dirty(),
        Command::CriticalPathAnalysis { variable_name } => {
            let graph = graph.read().unwrap();
            let mut r = DispatchResult::default().msg(format!(
                "performing critical path analysis using variable {}",
                variable_name
            ));
            let longest_path =
                find_longest_path(&graph, CostCalculator::new(variable_name.clone()));
            for (i, node) in longest_path.ids.iter().enumerate() {
                if let Some(label) = graph.find_node_label(node) {
                    let val = match graph.find_node_variable_value(node, &variable_name) {
                        Some(val) => format!("{}", val),
                        None => "".to_string(),
                    };
                    r = r.msg(format!("Step {}: {}: {}", i + 1, val, label));
                }
            }
            if !longest_path.ids.is_empty() {
                r = r.msg("====================");
                if let Some(cost) = longest_path.cost {
                    r = r.msg(format!("Total cost: {}", cost));
                }
                r = r.msg(format!("Total length: {}", longest_path.ids.len()));
            }
            r
        }
        Command::ParseError { .. } => {
            DispatchResult::default().msg("could not understand command; try 'h' for help")
        }
        Command::Exit => DispatchResult::default().quit(),
        Command::CostAnalysis { variable_name } => {
            let graph = graph.read().unwrap();
            let r = DispatchResult::default().msg(format!(
                "performing cost analysis using variable {}",
                variable_name
            ));
            let cost = find_cost(&graph, CostCalculator::new(variable_name.clone()));
            r.msg(format!("Total cost: {}", cost))
        }
    }
}
