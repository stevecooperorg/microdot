//! petgraph functions.
use crate::graph::{Graph, Node, VariableValue};
use crate::labels::NodeInfo;
use crate::Id;
use petgraph::algo::all_simple_paths;
use petgraph::prelude::NodeIndex;
use std::collections::BTreeMap;

pub trait GetVariableValue<T> {
    fn get_weight(&self, item: &T) -> Option<VariableValue>;
}

impl<T, F: Fn(&T) -> Option<VariableValue>> GetVariableValue<T> for F {
    fn get_weight(&self, item: &T) -> Option<VariableValue> {
        (self)(item)
    }
}

#[derive(Default)]
pub struct PGraph {
    pub graph: petgraph::Graph<f64, f64>,
    index_to_id: BTreeMap<NodeIndex, Id>,
    id_to_index: BTreeMap<Id, NodeIndex>,
}

impl PGraph {
    pub fn new() -> Self {
        PGraph::default()
    }

    pub fn add_node(&mut self, id: Id) -> petgraph::graph::NodeIndex {
        let idx = self.graph.add_node(1.0f64);
        self.id_to_index.insert(id.clone(), idx);
        self.index_to_id.insert(idx, id);
        idx
    }

    pub fn add_edge(&mut self, from: petgraph::graph::NodeIndex, to: petgraph::graph::NodeIndex) {
        self.graph.add_edge(from, to, Default::default());
    }
}

pub struct Path {
    pub ids: Vec<Id>,
    pub cost: Option<VariableValue>,
}

pub fn find_longest_path(
    graph: &Graph,
    get_weights: impl GetVariableValue<crate::graph::Node>,
) -> Path {
    find_path(graph, get_weights, false)
}

pub fn find_shortest_path(
    graph: &Graph,
    get_weights: impl GetVariableValue<crate::graph::Node>,
) -> Path {
    find_path(graph, get_weights, true)
}

fn find_path(
    graph: &Graph,
    get_weights: impl GetVariableValue<crate::graph::Node>,
    shortest: bool,
) -> Path {
    // convert our graph to a petgraph so we can use the algorithms;
    let use_negative_weights = !shortest; // if we're looking for the longest path, we're looking for the 'negative-shortest' path
    let pgraph = graph.to_petgraph();

    // we're going to gather every possible path through the graph, from all sources to
    // all targets, and then sort.
    let mut all_paths: Vec<Vec<Id>> = vec![];

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
                let pgraph_paths: Vec<Vec<NodeIndex>> =
                    all_simple_paths(&pgraph.graph, source_idx, sink_idx, 0, None).collect();
                for path in pgraph_paths {
                    let path: Vec<Id> = path
                        .into_iter()
                        .filter_map(|idx| pgraph.index_to_id.get(&idx))
                        .cloned()
                        .collect();
                    all_paths.push(path);
                }
            }
        }
    }

    let node_weights = graph.node_weights(get_weights);

    fn path_cost(
        path: &[Id],
        node_weights: &BTreeMap<Id, Option<VariableValue>>,
    ) -> Option<VariableValue> {
        let costs: Vec<VariableValue> = path
            .iter()
            .filter_map(|id| node_weights.get(id))
            .flatten()
            .cloned()
            .collect();
        if costs.is_empty() {
            None
        } else {
            Some(costs.into_iter().sum())
        }
    }

    // we now find the cheapest path by summing the weights of the nodes in the path. You can use a
    // negative weight function to find the longest path.
    all_paths.sort_by_key(|path| {
        let cost = path_cost(path, &node_weights).unwrap_or_else(VariableValue::zero);
        let len = path.len() as i32;
        // sort by cost but tie-break on length
        if use_negative_weights {
            (-cost, -len)
        } else {
            (cost, len)
        }
    });

    let path = if let Some(path) = all_paths.first() {
        path.to_vec()
    } else {
        vec![]
    };

    let cost = all_paths
        .first()
        .and_then(|path| path_cost(path, &node_weights));

    Path { ids: path, cost }
}

pub fn find_cost(
    graph: &Graph,
    get_weights: impl GetVariableValue<crate::graph::Node>,
) -> VariableValue {
    let node_weights = graph.node_weights(get_weights);
    node_weights.values().flatten().cloned().sum()
}

pub struct CostCalculator {
    variable_name: String,
}

impl CostCalculator {
    pub fn new(variable_name: impl Into<String>) -> Self {
        CostCalculator {
            variable_name: variable_name.into(),
        }
    }
}

impl GetVariableValue<Node> for CostCalculator {
    fn get_weight(&self, item: &Node) -> Option<VariableValue> {
        let NodeInfo { variables, .. } = NodeInfo::parse(item.label());
        variables.get(&self.variable_name).map(|v| v.value.clone())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::graph::Time;
    use crate::Label;

    fn uniform_weight<T>(_node: &T) -> Option<VariableValue> {
        Some(VariableValue::number(1.0))
    }

    #[test]
    pub fn shortest_path_can_be_found() {
        let mut graph = Graph::new();
        let a = graph.insert_node(Label("A".to_string())).0;
        let b = graph.insert_node(Label("B".to_string())).0;
        let c = graph.insert_node(Label("C".to_string())).0;
        graph.link_edge(&a, &b);
        graph.link_edge(&b, &c);

        let path = find_shortest_path(&graph, uniform_weight);
        assert_eq!(path.ids, vec![a, b, c]);
        assert_eq!(path.cost, Some(VariableValue::number(3.0)));
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
        fn s1_is_expensive(node: &crate::graph::Node) -> Option<VariableValue> {
            if node.label().0 == "slow1" {
                Some(VariableValue::number(-10.0))
            } else {
                Some(VariableValue::number(-1.0))
            }
        }
        let path = find_shortest_path(&graph, s1_is_expensive);
        assert_eq!(path.ids, vec![q1, s1, q4]);
        assert_eq!(path.cost, Some(VariableValue::number(-12.0)));
    }

    #[test]
    pub fn shortest_and_longest_paths_can_be_found_via_a_variable() {
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

        let longest_path = find_longest_path(&graph, CostCalculator::new("cost"));
        assert_eq!(longest_path.ids, vec![q1.clone(), s1, q4.clone()]);
        assert_eq!(
            longest_path.cost,
            Some(VariableValue::time(Time::Minute(500)))
        );

        let shortest_path = find_shortest_path(&graph, CostCalculator::new("cost"));
        assert_eq!(shortest_path.ids, vec![q1, q2, q3, q4]);
        assert_eq!(
            shortest_path.cost,
            Some(VariableValue::time(Time::Minute(40)))
        );
    }

    #[test]
    pub fn shortest_and_longest_paths_based_on_length_if_no_variables() {
        let mut graph = Graph::new();
        // create a graph with a short path (q1 -> q4) and a long path (q1 -> q2 -> q3 -> q4)
        let q1 = graph.insert_node(Label("q1".to_string())).0;
        let q2 = graph.insert_node(Label("q2".to_string())).0;
        let q3 = graph.insert_node(Label("q3".to_string())).0;
        let q4 = graph.insert_node(Label("q4".to_string())).0;

        // short path q1 -> s1 -> q4
        graph.link_edge(&q1, &q4);

        // long path q1 -> q2 -> q3 -> q4
        graph.link_edge(&q1, &q2);
        graph.link_edge(&q2, &q3);
        graph.link_edge(&q3, &q4);

        let shortest = find_shortest_path(&graph, CostCalculator::new("cost"));
        assert_eq!(shortest.ids, vec![q1.clone(), q4.clone()]);
        assert_eq!(shortest.cost, None);

        let longest = find_longest_path(&graph, CostCalculator::new("cost"));
        assert_eq!(longest.ids, vec![q1, q2, q3, q4]);
        assert_eq!(longest.cost, None);
    }

    #[test]
    pub fn handles_empty_cost_nodes_gracefully() {
        let mut graph = Graph::new();
        // create a graph with some nodes that don't have costs
        // this is 20m explicit and some unknown value for the others
        let q1 = graph.insert_node(Label("quick1".to_string())).0;
        let q2 = graph.insert_node(Label("quick2 $cost=10m".to_string())).0;
        let q3 = graph.insert_node(Label("quick3".to_string())).0;
        let q4 = graph.insert_node(Label("quick4 $cost=10m".to_string())).0;

        // only path q1 -> q2 -> q3 -> q4
        graph.link_edge(&q1, &q2);
        graph.link_edge(&q2, &q3);
        graph.link_edge(&q3, &q4);

        let shortest_path = find_shortest_path(&graph, CostCalculator::new("cost"));
        assert_eq!(shortest_path.ids, vec![q1, q2, q3, q4]);
        assert_eq!(
            shortest_path.cost,
            Some(VariableValue::time(Time::Minute(20)))
        );
    }
}
