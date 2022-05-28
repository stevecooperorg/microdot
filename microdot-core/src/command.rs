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
