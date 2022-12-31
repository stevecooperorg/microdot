use crate::graphviz::DisplayMode;
use fdg_img::{
    style::{
        text_anchor::{HPos, Pos, VPos},
        Color, IntoFont, RGBAColor, TextStyle, BLACK,
    },
    Settings,
};
use fdg_sim::glam::Vec3;
use fdg_sim::petgraph::stable_graph::NodeIndex;
use fdg_sim::{ForceGraph, ForceGraphHelper};
use microdot_core::exporter::{Exporter, NodeHighlight};
use microdot_core::graph::Graph;
use microdot_core::{Id, Label};
use std::collections::HashMap;

pub struct FdgExporter {
    inner_content: ForceGraph<(), ()>,
    node_map: HashMap<Id, NodeIndex>,
    is_left_right: bool,
    display_mode: DisplayMode,
}

impl Default for FdgExporter {
    fn default() -> Self {
        Self {
            inner_content: ForceGraph::default(),
            node_map: HashMap::new(),
            is_left_right: true,
            display_mode: DisplayMode::Interactive,
        }
    }
}

impl FdgExporter {
    pub fn export(self, graph: &Graph) -> String {
        let mut this = self;
        graph.export(&mut this);

        // stick on 'start' and 'end' nodes;

        let start_coords = Vec3 {
            x: -100.0,
            y: 0.0,
            z: 0.0,
        };
        let end_coords = Vec3 {
            x: 100.0,
            y: 0.0,
            z: 0.0,
        };
        let start_ni = this
            .inner_content
            .add_force_node_with_coords("start", (), start_coords);
        this.inner_content[start_ni].locked = true;

        // link all sources to the start NI
        //for node in graph.

        let end_ni = this
            .inner_content
            .add_force_node_with_coords("end", (), end_coords);
        this.inner_content[end_ni].locked = true;

        let FdgExporter {
            inner_content,
            node_map,
            is_left_right,
            display_mode,
        } = this;
        // generate svg text for your graph
        let g: fdg_sim::ForceGraph<(), ()> = inner_content;

        let text_style = Some(TextStyle {
            font: ("sans-serif", 20).into_font(),
            color: BLACK.to_backend_color(),
            pos: Pos {
                h_pos: HPos::Left,
                v_pos: VPos::Center,
            },
        });

        let settings = Some(Settings {
            text_style,
            node_color: RGBAColor(100, 100, 100, 1.0),
            edge_color: RGBAColor(150, 150, 150, 1.0),
            ..Default::default()
        });

        fdg_img::gen_image(g, settings).unwrap()
    }
}

impl Exporter for FdgExporter {
    fn set_direction(&mut self, is_left_right: bool) {
        self.is_left_right = is_left_right;
    }

    fn add_node(&mut self, id: &Id, label: &Label, _highlight: NodeHighlight) {
        let ni = self.inner_content.add_force_node(label.to_string(), ());

        self.node_map.insert(id.clone(), ni);
    }

    fn add_edge(&mut self, id: &Id, from: &Id, to: &Id) {
        let from_ni = self.node_map[from];
        let to_ni = self.node_map[to];
        self.inner_content.add_edge(from_ni, to_ni, ());
    }
}
