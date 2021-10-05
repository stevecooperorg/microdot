pub mod graph;
pub mod graphviz;
pub mod json;
pub mod parser;

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

pub trait Exporter {
    fn set_direction(&mut self, is_left_right: bool);

    fn add_node(&mut self, id: &Id, label: &Label, highlight: bool);

    fn add_edge(&mut self, id: &Id, from: &Id, to: &Id);
}
