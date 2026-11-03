//! Persist JSON/DOT and optionally compile SVG.

use crate::graphviz::{self, DisplayMode, GraphVizExporter};
use crate::json::JsonExporter;
use crate::util::write_if_different;
use anyhow::{anyhow, Result};
use microdot_core::graph::Graph;
use microdot_core::CommandResult;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::UnboundedSender;

/// Write JSON + DOT beside `json_file`. When `compile_svg` is true, run Graphviz
/// and optionally notify a live-preview channel.
pub fn persist_graph(
    json_file: &Path,
    graph: &Graph,
    compile_svg: bool,
    reload_tx: Option<&UnboundedSender<()>>,
) -> Result<String> {
    let interactive_dot_file = save_dot_file(json_file, graph)?;
    if !compile_svg {
        return Ok(format!(
            "saved json/dot: {}",
            interactive_dot_file.to_string_lossy()
        ));
    }
    Ok(compile_dot(interactive_dot_file, reload_tx).to_string())
}

/// Persist from shared graph lock (acquires write lock).
pub fn persist_locked(
    json_file: &Path,
    graph: &Arc<RwLock<Graph>>,
    compile_svg: bool,
    reload_tx: Option<&UnboundedSender<()>>,
) -> Result<String> {
    let graph = match graph.write() {
        Ok(graph) => graph,
        Err(e) => return Err(anyhow!(e.to_string())),
    };
    persist_graph(json_file, &graph, compile_svg, reload_tx)
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

fn compile_dot(
    interactive_dot_file: PathBuf,
    reload_tx: Option<&UnboundedSender<()>>,
) -> CommandResult {
    let svg_compile = graphviz::compile(&interactive_dot_file);

    if svg_compile.is_ok() {
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
