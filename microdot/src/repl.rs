use crate::dispatch::dispatch_command;
use crate::parser::parse_line;
use crate::persist::persist_locked;
use crate::Interaction;
use anyhow::Result;
use microdot_core::graph::Graph;
use microdot_core::Line;
use rustyline::error::ReadlineError;
use std::path::Path;
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
        let msg = persist_locked(
            json_file,
            &graph,
            interaction.should_compile(),
            Some(&reload_tx),
        )?;
        if interaction.should_compile() {
            interaction.log(msg);
        }

        let dirty = match readline {
            Ok(line) => {
                interaction.add_history(line.as_str());

                let line = Line::new(line);
                let command = parse_line(line);
                let result = dispatch_command(command, &graph, json_file);
                for m in &result.messages {
                    interaction.log(m);
                }
                if result.quit {
                    return Ok(());
                }
                result.dirty
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
            let msg = persist_locked(
                json_file,
                &graph,
                interaction.should_compile(),
                Some(&reload_tx),
            )?;
            if interaction.should_compile() {
                interaction.log(msg);
            }
        }
    }
}
