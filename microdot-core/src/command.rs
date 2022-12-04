use crate::{Id, Label};

#[derive(PartialEq, Debug)]
pub enum GraphCommand {
    DeleteNode { id: Id },
    ExpandEdge { id: Id, label: Label },
    InsertAfterNode { id: Id, label: Label },
    InsertBeforeNode { id: Id, label: Label },
    InsertNode { label: Label },
    LinkEdge { from: Id, to: Id },
    RenameNode { id: Id, label: Label },
    SelectNode { id: Id },
    SetDirection { is_left_right: bool },
    UnlinkEdge { id: Id },
}

impl GraphCommand {
    pub fn to_help_string(&self) -> String {
        match self {
            GraphCommand::DeleteNode { id } => format!("Delete the <{}> node", id),
            GraphCommand::ExpandEdge { id, label } => format!(
                "Expand the <{}> edge with a new node labelled \"{}\"",
                id, label
            ),
            GraphCommand::InsertAfterNode { id, label } => format!(
                "Insert a node labelled \"{}\" after the node with id \"{}\"",
                label, id
            ),
            GraphCommand::InsertBeforeNode { id, label } => format!(
                "Insert a node labelled \"{}\" before the node with id \"{}\"",
                label, id
            ),
            GraphCommand::InsertNode { label } => {
                format!("Insert a node labelled \"{}\" into the graph", label)
            }
            GraphCommand::LinkEdge { from, to } => {
                format!("Link the <{}> node to the <{}> node", from, to)
            }
            GraphCommand::RenameNode { id, label } => {
                format!("Rename the <{}> node to \"{}\"", id, label)
            }
            GraphCommand::SelectNode { id } => format!("Select the <{}> node and highlight it", id),
            GraphCommand::SetDirection { is_left_right } => format!(
                "Change the orientation of the graph to {}",
                if *is_left_right {
                    "left to right"
                } else {
                    "top to bottom"
                }
            ),
            GraphCommand::UnlinkEdge { id } => format!("Unlink the <{}> edge", id),
        }
    }
}
