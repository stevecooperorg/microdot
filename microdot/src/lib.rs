use microdot_core::command::GraphCommand;
use microdot_core::{Id, Label, Line};
use rustyline::{Editor, Helper};

pub mod colors;
pub mod graphviz;
pub mod helper;
pub mod json;
pub mod palettes;
pub mod parser;
pub mod repl;
pub mod svg;

#[derive(PartialEq, Debug)]
pub enum Command {
    GraphCommand(GraphCommand),
    ShowHelp,
    Search { sub_label: Label },
    PrintDot,
    PrintJson,
    RenameNodeUnlabelled { id: Id },
    Save,
    Show,
    Exit,
    ParseError { line: Line },
}

impl From<GraphCommand> for Command {
    fn from(c: GraphCommand) -> Self {
        Command::GraphCommand(c)
    }
}

// a trait which deals with the R & P of REPL: Read and Print; can be mixed in with a loop
pub trait Interaction {
    fn read(&mut self, prompt: &str) -> rustyline::Result<String>;
    fn add_history<S: AsRef<str> + Into<String>>(&mut self, history: S) -> bool;
    fn log<S: AsRef<str> + Into<String>>(&mut self, message: S);
    fn should_compile_dot(&self) -> bool;
}

impl<H> Interaction for Editor<H>
where
    H: Helper,
{
    fn read(&mut self, prompt: &str) -> rustyline::Result<String> {
        self.readline(prompt)
    }

    fn add_history<S: AsRef<str> + Into<String>>(&mut self, history: S) -> bool {
        self.add_history_entry(history)
    }

    fn log<S: AsRef<str> + Into<String>>(&mut self, message: S) {
        println!("{}", message.into());
    }

    fn should_compile_dot(&self) -> bool {
        true
    }
}
