use petgraph::dot::{Config, Dot};
use petgraph::graph::UnGraph;

/// Wrapper for petgraph::dot::Dot.
/// It actually shows real nodes' ids.
pub fn to_dot_str(graph: &UnGraph<u32, String>) -> String {
    Dot::with_config(graph, &[Config::NodeIndexLabel, Config::EdgeNoLabel]).to_string()
}

/// Writes the graph to a file in DOT format.
pub fn to_dot_file(graph: &UnGraph<u32, String>, path: &str) {
    let dot_str = to_dot_str(graph);
    std::fs::write(path, dot_str).expect("Rust shoudl write to file");
}

/// Writes a string to a file.
pub fn to_file(content: &str, path: &str) {
    std::fs::write(path, content).expect("Rust should write to file");
}
