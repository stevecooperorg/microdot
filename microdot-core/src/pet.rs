//! petgraph functions.
use crate::graph::{Graph, Node, VariableValue};
use crate::labels::NodeInfo;
use crate::Id;
use petgraph::algo::all_simple_paths;
use petgraph::prelude::NodeIndex;
use std::collections::BTreeMap;

type Weight = i32;

pub trait GetWeight<T> {
    fn get_weight(&self, item: &T) -> Weight;
}

impl<T, F: Fn(&T) -> Weight> GetWeight<T> for F {
    fn get_weight(&self, item: &T) -> Weight {
        (self)(item)
    }
}

#[derive(Default)]
pub struct PGraph {
    pub graph: petgraph::Graph<Weight, Weight>,
    index_to_id: BTreeMap<NodeIndex, Id>,
    id_to_index: BTreeMap<Id, NodeIndex>,
}

impl PGraph {
    pub fn new() -> Self {
        PGraph::default()
    }

    pub fn add_node(&mut self, id: Id, weight: Weight) -> petgraph::graph::NodeIndex {
        let idx = self.graph.add_node(weight);
        self.id_to_index.insert(id.clone(), idx);
        self.index_to_id.insert(idx, id);
        idx
    }

    pub fn add_edge(&mut self, from: petgraph::graph::NodeIndex, to: petgraph::graph::NodeIndex) {
        self.graph.add_edge(from, to, 0);
    }

    pub fn find_node_weight(&self, id: petgraph::graph::NodeIndex) -> Option<Weight> {
        self.graph.node_weight(id).cloned()
    }
}

pub struct Path {
    pub ids: Vec<Id>,
    pub cost: String,
}

pub fn find_shortest_path(graph: &Graph, get_weights: impl GetWeight<crate::graph::Node>) -> Path {
    // convert our graph to a petgraph so we can use the algorithms;
    let pgraph = graph.to_petgraph(get_weights);

    // we're going to gather every possible path through the graph, from all sources to
    // all targets, and then sort.
    let mut all_paths: Vec<Vec<NodeIndex>> = vec![];

    let sources = graph.sources();
    let sinks = graph.sinks();

    for source in sources {
        for sink in &sinks {
            if let (Some(source_idx), Some(sink_idx)) = (
                pgraph.id_to_index.get(&source),
                pgraph.id_to_index.get(sink),
            ) {
                // calculate all the paths from the source to the sink and add it to the collection.
                let source_idx = *source_idx;
                let sink_idx = *sink_idx;
                let mut paths: Vec<Vec<NodeIndex>> =
                    all_simple_paths(&pgraph.graph, source_idx, sink_idx, 0, None).collect();
                all_paths.append(&mut paths);
            }
        }
    }

    // we now find the cheapest path by summing the weights of the nodes in the path. You can use a
    // negative weight function to find the longest path.
    all_paths.sort_by_key(|path| {
        path.iter()
            .map(|idx| pgraph.find_node_weight(*idx).unwrap())
            .sum::<Weight>()
    });

    let path = if let Some(path) = all_paths.first() {
        let best_path = path
            .iter()
            .map(|idx| pgraph.index_to_id[idx].clone())
            .collect();
        best_path
    } else {
        vec![]
    };

    Path {
        ids: path,
        cost: "unimplemented".to_string(),
    }
}

pub struct CostCalculator {
    variable_name: String,
    find_longest: bool,
}

impl CostCalculator {
    pub fn new(variable_name: impl Into<String>, find_longest: bool) -> Self {
        CostCalculator {
            variable_name: variable_name.into(),
            find_longest,
        }
    }
}

impl GetWeight<Node> for CostCalculator {
    fn get_weight(&self, item: &Node) -> Weight {
        let NodeInfo { variables, .. } = NodeInfo::parse(item.label());
        let cost = if let Some(cost) = variables.get(&self.variable_name) {
            match &cost.value {
                VariableValue::Number(n) => *n as i32,
                VariableValue::Time(t) => t.to_minutes() as i32,
                _ => 1,
            }
        } else {
            1
        };
        if self.find_longest {
            -cost
        } else {
            cost
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::Label;

    fn uniform_weight<T>(_node: &T) -> Weight {
        1
    }

    #[test]
    pub fn shortest_path_can_be_found() {
        let mut graph = Graph::new();
        let a = graph.insert_node(Label("A".to_string())).0;
        let b = graph.insert_node(Label("B".to_string())).0;
        let c = graph.insert_node(Label("C".to_string())).0;
        graph.link_edge(&a, &b);
        graph.link_edge(&b, &c);

        let path = find_shortest_path(&graph, uniform_weight).ids;
        assert_eq!(path, vec![a, b, c]);
    }

    #[test]
    pub fn shortest_path_can_be_found_with_expensive_node() {
        let mut graph = Graph::new();
        // create a graph with a quick path and a slow path -- the quick path has
        // more nodes in it, but the slow path is more expensive.
        let q1 = graph.insert_node(Label("quick1".to_string())).0;
        let s1 = graph.insert_node(Label("slow1".to_string())).0;
        let q2 = graph.insert_node(Label("quick2".to_string())).0;
        let q3 = graph.insert_node(Label("quick3".to_string())).0;
        let q4 = graph.insert_node(Label("quick4".to_string())).0;

        // slow path q1 -> s1 -> q4
        graph.link_edge(&q1, &s1);
        graph.link_edge(&s1, &q4);

        // fast path q1 -> q2 -> q3 -> q4
        graph.link_edge(&q1, &q2);
        graph.link_edge(&q2, &q3);
        graph.link_edge(&q3, &q4);

        // we're finding the _longest_ path by using a negative weight function
        fn s1_is_expensive(node: &crate::graph::Node) -> Weight {
            if node.label().0 == "slow1" {
                -10
            } else {
                -1
            }
        }
        let path = find_shortest_path(&graph, s1_is_expensive).ids;
        assert_eq!(path, vec![q1, s1, q4]);
    }

    #[test]
    pub fn shortest_path_can_be_found_via_a_variable() {
        let mut graph = Graph::new();
        // create a graph with a quick path and a slow path -- the quick path has
        // more nodes in it, but the slow path is more expensive.
        let q1 = graph.insert_node(Label("quick1 $cost=10m".to_string())).0;
        let s1 = graph.insert_node(Label("slow1 $cost=1d".to_string())).0;
        let q2 = graph.insert_node(Label("quick2 $cost=10m".to_string())).0;
        let q3 = graph.insert_node(Label("quick3 $cost=10m".to_string())).0;
        let q4 = graph.insert_node(Label("quick4 $cost=10m".to_string())).0;

        // slow path q1 -> s1 -> q4
        graph.link_edge(&q1, &s1);
        graph.link_edge(&s1, &q4);

        // fast path q1 -> q2 -> q3 -> q4
        graph.link_edge(&q1, &q2);
        graph.link_edge(&q2, &q3);
        graph.link_edge(&q3, &q4);

        let longest_path = find_shortest_path(&graph, CostCalculator::new("cost", true)).ids;
        assert_eq!(longest_path, vec![q1.clone(), s1, q4.clone()]);

        let shortest_path = find_shortest_path(&graph, CostCalculator::new("cost", false)).ids;
        assert_eq!(shortest_path, vec![q1, q2, q3, q4]);
    }
}
