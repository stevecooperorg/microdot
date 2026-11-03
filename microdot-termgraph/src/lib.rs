//! Terminal graph renderer for microdot.
//!
//! Drawing routines are adapted from [ascii-petgraph](https://github.com/elefthei/ascii-petgraph)
//! (MIT). This crate does **not** perform layout — supply absolute character-cell
//! positions (typically derived from Graphviz `-Tplain`).

pub mod grid;
pub mod render;
pub mod scene;
pub mod style;
pub mod widget;

pub use grid::{Cell, CharGrid};
pub use render::{display_label, node_box_width, render_scene, NODE_BOX_HEIGHT};
pub use scene::{LabelMode, PlacedEdge, PlacedNode, Scene};
pub use style::{BoxBorder, EdgeChars, EdgeStyle, NodeStyle};
pub use widget::SceneWidget;
