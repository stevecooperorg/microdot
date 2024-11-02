use microdot_core::command::GraphCommand;
use microdot_core::{Id, Label, Line};
use rustyline::history::History;
use rustyline::{Editor, Helper};

pub mod filters;
pub mod graphviz;
pub mod helper;
pub mod json;
pub mod parser;
pub mod repl;
// mod storage;
pub mod svg;
pub mod util;
pub mod web;

#[derive(PartialEq, Eq, Debug)]
pub enum Command {
    GraphCommand(GraphCommand),
    ShowHelp,
    Search { sub_label: Label },
    PrintDot,
    PrintJson,
    RenameNodeUnlabelled { id: Id },
    Save,
    CriticalPathAnalysis { variable_name: String },
    CostAnalysis { variable_name: String },
    Show,
    Exit,
    ParseError { line: Line },
}

impl Command {
    #[allow(dead_code)]
    fn to_help_string(&self) -> String {
        match self {
            Command::GraphCommand(c) => c.to_help_string(),
            Command::ShowHelp => "display this help".into(),
            Command::Search { sub_label } => {
                format!("search for <{}> and highlight matching nodes", sub_label)
            }
            Command::PrintDot => "print the dot definition for this graph to the terminal".into(),
            Command::PrintJson => "print the json definition for this graph to the terminal".into(),
            Command::RenameNodeUnlabelled { id } => {
                format!("rename <{}> but no new label text supplied", id)
            }
            Command::Save => "save the graph to disc".into(),
            Command::CriticalPathAnalysis { variable_name } => format!(
                "do a critical path analysis on the graph using <{}> as the cost",
                variable_name
            ),
            Command::Show => "open the diagram in Gapplin".into(),
            Command::Exit => "exit microdot".into(),
            Command::ParseError { line } => format!("could not parse: \"{}\"", line),
            Command::CostAnalysis { variable_name } => format!(
                "sum the cost of all nodes in the grpa using <{}> as the cost",
                variable_name
            ),
        }
    }
}

impl From<GraphCommand> for Command {
    fn from(c: GraphCommand) -> Self {
        Command::GraphCommand(c)
    }
}

// a trait which deals with the R & P of REPL: Read and Print; can be mixed in with a loop
pub trait Interaction {
    // TODO: this makes more sense as something like a stream of input lines, with an EOF from
    // readline => None.
    fn read(&mut self, prompt: &str) -> rustyline::Result<String>;
    // TODO: should this be converted to a futures::sink::Sink? 'write to the history channel'
    fn add_history<S: AsRef<str> + Into<String>>(&mut self, history: S) -> bool;
    // TODO: possibly another futures::sink::Sink
    fn log<S: AsRef<str> + Into<String>>(&mut self, message: S);
    // TODO: bad design. Should be handled outside; really corresponds to 'did the last command
    // dirty the cache'
    fn should_compile(&self) -> bool;
}

impl<H, I> Interaction for Editor<H, I>
where
    H: Helper,
    I: History,
{
    fn read(&mut self, prompt: &str) -> rustyline::Result<String> {
        self.readline(prompt)
    }

    fn add_history<S: AsRef<str> + Into<String>>(&mut self, history: S) -> bool {
        self.add_history_entry(history).unwrap_or(false)
    }

    fn log<S: AsRef<str> + Into<String>>(&mut self, message: S) {
        println!("{}", message.into());
    }

    fn should_compile(&self) -> bool {
        true
    }
}
