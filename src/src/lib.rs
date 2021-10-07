use rustyline::Editor;

pub mod graph;
pub mod graphviz;
pub mod json;
pub mod parser;
pub mod repl;

macro_rules! new_string_type {
    ($id: ident) => {
        #[derive(PartialEq, Eq, Hash, Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $id(String);

        impl $id {
            pub fn new<S: Into<String>>(str: S) -> Self {
                Self(str.into())
            }
        }

        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

new_string_type!(CommandResult);
new_string_type!(Label);
new_string_type!(Id);
new_string_type!(Line);

#[derive(PartialEq, Debug)]
pub enum Command {
    GraphCommand(GraphCommand),
    ShowHelp,
    Search { sub_label: Label },
    PrintDot,
    PrintJson,
    Save,
    Exit,
    ParseError { line: Line },
}

#[derive(PartialEq, Debug)]
pub enum GraphCommand {
    InsertNode { label: Label },
    DeleteNode { id: Id },
    LinkEdge { from: Id, to: Id },
    RenameNode { id: Id, label: Label },
    InsertAfterNode { id: Id, label: Label },
    InsertBeforeNode { id: Id, label: Label },
    ExpandEdge { id: Id, label: Label },
    UnlinkEdge { id: Id },
    SetDirection { is_left_right: bool },
}

impl From<GraphCommand> for Command {
    fn from(c: GraphCommand) -> Self {
        Command::GraphCommand(c)
    }
}

#[derive(Copy, Clone)]
pub enum NodeHighlight {
    Normal,
    SearchResult,
    CurrentNode,
}

pub trait Exporter {
    fn set_direction(&mut self, is_left_right: bool);

    fn add_node(&mut self, id: &Id, label: &Label, highlight: NodeHighlight);

    fn add_edge(&mut self, id: &Id, from: &Id, to: &Id);
}

// a trait which deals with the R & P of REPL: Read and Print; can be mixed in with a loop
pub trait Interaction {
    fn read(&mut self, prompt: &str) -> rustyline::Result<String>;
    fn add_history<S: AsRef<str> + Into<String>>(&mut self, history: S) -> bool;
    fn log<S: AsRef<str> + Into<String>>(&mut self, message: S);
    fn should_compile_dot(&self) -> bool;
}

impl Interaction for Editor<()> {
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
