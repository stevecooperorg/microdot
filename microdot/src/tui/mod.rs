//! Full-screen ratatui editor for microdot graphs.

mod app;
mod draw;
mod keys;
mod menu;
mod theme;
mod viz;

pub use theme::{Theme, ThemeKind};

use app::App;
use anyhow::{Context, Result};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use microdot_core::graph::Graph;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

struct Restore;

impl Drop for Restore {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
    }
}

/// Run the TUI until quit. Blocks the calling thread.
pub fn run(
    graph: Arc<RwLock<Graph>>,
    json_file: PathBuf,
    reload_tx: UnboundedSender<()>,
    theme: Theme,
) -> Result<()> {
    enable_raw_mode().context("raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).context("alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("terminal")?;
    let _restore = Restore;

    let mut app = App::new(graph, json_file, reload_tx, theme);
    // Initial persist so SVG/json exist for optional --port preview.
    app.persist(true);

    let result = event_loop(&mut terminal, &mut app);
    // Save on clean exit if still dirty.
    if app.dirty {
        app.persist(true);
    }
    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<()> {
    let mut dirty_ticks: u32 = 0;
    loop {
        terminal.draw(|frame| draw::draw(frame, app)).context("draw")?;

        if !event::poll(Duration::from_millis(200)).context("poll")? {
            if app.dirty {
                dirty_ticks += 1;
                // Debounce: persist JSON/DOT (and SVG) after ~1s idle while dirty.
                if dirty_ticks >= 5 {
                    app.persist(true);
                    dirty_ticks = 0;
                }
            }
            continue;
        }
        dirty_ticks = 0;

        match event::read().context("read")? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                if keys::handle_key(app, key) {
                    break;
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
    Ok(())
}
