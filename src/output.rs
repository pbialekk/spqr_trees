use crate::UnGraph;
use petgraph::dot::{Config, Dot};

/// Wrapper for petgraph::dot::Dot.
///
/// It shows your nodes labels, not petgraph's internal indices.
///
/// It adds colors also.
///
/// Real edges are solid and virtual edges are dashed.
pub fn to_dot_str(graph: &UnGraph) -> String {
// TODO: add case when we have virtual edges
    Dot::with_attr_getters(
        graph,
        &[Config::EdgeNoLabel, Config::NodeNoLabel],
        &|_, edge_ref| format!("label=\"{}\"", edge_ref.weight()),
        &|g, node_ref| {
            format!(
                "label=\"{}\", style=filled, fillcolor=lightblue",
                g.node_weight(node_ref.0).unwrap()
            )
        },
    )
    .to_string()
}

/// Writes the graph to a file in DOT format.
pub fn to_dot_file(graph: &UnGraph, path: &str) {
    let dot_str = to_dot_str(graph);
    to_file(&dot_str, path);
}

/// Writes a string to a file.
pub fn to_file(content: &str, path: &str) {
    std::fs::write(path, content).expect("Rust should write to file");
}
