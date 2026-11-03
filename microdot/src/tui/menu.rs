//! Menubar actions for the microdot TUI.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuGroup {
    File,
    Edit,
    Graph,
    View,
    Help,
}

impl MenuGroup {
    pub const ALL: [MenuGroup; 5] = [
        MenuGroup::File,
        MenuGroup::Edit,
        MenuGroup::Graph,
        MenuGroup::View,
        MenuGroup::Help,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MenuGroup::File => "File",
            MenuGroup::Edit => "Edit",
            MenuGroup::Graph => "Graph",
            MenuGroup::View => "View",
            MenuGroup::Help => "Help",
        }
    }

    pub fn items(self) -> &'static [MenuAction] {
        match self {
            MenuGroup::File => &[
                MenuAction::Save,
                MenuAction::ShowSvg,
                MenuAction::PrintDot,
                MenuAction::PrintJson,
                MenuAction::Quit,
            ],
            MenuGroup::Edit => &[
                MenuAction::Insert,
                MenuAction::After,
                MenuAction::Before,
                MenuAction::Rename,
                MenuAction::Delete,
                MenuAction::DeleteKeepEdges,
                MenuAction::Link,
                MenuAction::Unlink,
                MenuAction::Expand,
            ],
            MenuGroup::Graph => &[
                MenuAction::DirLr,
                MenuAction::DirRl,
                MenuAction::DirTb,
                MenuAction::DirBt,
                MenuAction::CriticalPath,
                MenuAction::Cost,
            ],
            MenuGroup::View => &[
                MenuAction::Relayout,
                MenuAction::TruncateLabels,
                MenuAction::NumericIds,
            ],
            MenuGroup::Help => &[MenuAction::KeyHelp],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    Save,
    ShowSvg,
    PrintDot,
    PrintJson,
    Quit,
    Insert,
    After,
    Before,
    Rename,
    Delete,
    DeleteKeepEdges,
    Link,
    Unlink,
    Expand,
    DirLr,
    DirRl,
    DirTb,
    DirBt,
    CriticalPath,
    Cost,
    Relayout,
    TruncateLabels,
    NumericIds,
    KeyHelp,
}

impl MenuAction {
    pub fn label(self) -> &'static str {
        match self {
            MenuAction::Save => "Save",
            MenuAction::ShowSvg => "Show SVG",
            MenuAction::PrintDot => "Print DOT",
            MenuAction::PrintJson => "Print JSON",
            MenuAction::Quit => "Quit  q",
            MenuAction::Insert => "Insert node  i",
            MenuAction::After => "Insert after  a",
            MenuAction::Before => "Insert before  b",
            MenuAction::Rename => "Rename  r",
            MenuAction::Delete => "Delete  x",
            MenuAction::DeleteKeepEdges => "Delete keep edges  X",
            MenuAction::Link => "Link  l",
            MenuAction::Unlink => "Unlink  u",
            MenuAction::Expand => "Expand edge  e",
            MenuAction::DirLr => "Direction LR",
            MenuAction::DirRl => "Direction RL",
            MenuAction::DirTb => "Direction TB",
            MenuAction::DirBt => "Direction BT",
            MenuAction::CriticalPath => "Critical path…",
            MenuAction::Cost => "Cost…",
            MenuAction::Relayout => "Relayout viz",
            MenuAction::TruncateLabels => "Truncate labels",
            MenuAction::NumericIds => "Numeric ids",
            MenuAction::KeyHelp => "Key reference",
        }
    }
}
