use crate::{Direction, Id, Label};

#[derive(Copy, Clone)]
pub enum NodeHighlight {
    Normal,
    SearchResult,
    CurrentNode,
}

pub trait Exporter {
    fn set_direction(&mut self, direction: Direction);

    fn add_node(&mut self, id: &Id, label: &Label, highlight: NodeHighlight);

    fn add_edge(&mut self, id: &Id, from: &Id, to: &Id);
}
