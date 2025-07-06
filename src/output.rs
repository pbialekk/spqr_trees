use crate::UnGraph;
use petgraph::visit::EdgeRef;

/// Returns a graph in DOT format.
///
/// It shows your nodes labels, not petgraph's internal indices.
///
/// It adds colors also.
///
/// Real edges and structure edges are solid and virtual edges are dashed.
///
/// Intended to be used with `neato`.
pub fn draw_graph(graph: &UnGraph) -> String {
    let mut output = String::from("graph {\n");
    output.push_str("  mode=sgd;\n");
    output.push_str("  maxiter=1000;\n");
    output.push_str("  node [shape=circle, style=filled, fillcolor=lightblue];\n");
    
    // Add vertices
    for node_idx in graph.node_indices() {
        let label = graph.node_weight(node_idx).unwrap();
        output.push_str(&format!(
            "  {} [label=\"{}\"];\n",
            node_idx.index(),
            label
        ));
    }
    
    // Add edges
    for edge in graph.edge_references() {
        let (a, b) = (edge.source().index(), edge.target().index());
        let style = if *edge.weight() == crate::EdgeLabel::Virtual {
            "dashed"
        } else {
            "solid"
        };
        output.push_str(&format!(
            "  {} -- {} [style={}];\n",
            a, b, style
        ));
    }
    output.push_str("}\n");
    output
}

/// Writes the graph to a file in DOT format.
pub fn to_dot_file(graph: &UnGraph, path: &str) {
    let dot_str = draw_graph(graph);
    to_file(&dot_str, path);
}

/// Writes a string to a file.
pub fn to_file(content: &str, path: &str) {
    std::fs::write(path, content).expect("Rust should write to file");
}
