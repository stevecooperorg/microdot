pub mod graph;
pub mod graphviz;
pub mod json;
pub mod parser;

macro_rules! new_string_type {
    ($id: ident) => {
        #[derive(PartialEq, Debug, Clone)]

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
    PrintDot,
    PrintJson,
    Exit,
    ParseError { line: Line },
}

#[derive(PartialEq, Debug)]
pub enum GraphCommand {
    InsertNode { label: Label },
    DeleteNode { id: Id },
    LinkEdge { from: Id, to: Id },
    RenameNode { id: Id, label: Label },
    UnlinkEdge { id: Id },
}

impl From<GraphCommand> for Command {
    fn from(c: GraphCommand) -> Self {
        Command::GraphCommand(c)
    }
}

pub trait Exporter {
    fn add_node(&mut self, id: &Id, label: &Label);

    fn add_edge(&mut self, from: &Id, to: &Id);
}
