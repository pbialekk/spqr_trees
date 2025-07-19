use petgraph::graph::NodeIndex;
use crate::{EdgeLabel, UnGraph};

/// Generates a grid graph with the specified number of rows and columns.
pub(crate) fn generate_grid_graph(rows: usize, cols: usize) -> UnGraph {
    let mut graph = UnGraph::new_undirected();

    for r in 0..rows {
        for c in 0..cols {
            graph.add_node((r * cols + c) as u32);
        }
    }

    for r in 0..rows {
        for c in 0..cols {
            if r + 1 < rows {
                graph.add_edge(NodeIndex::new(r * cols + c), NodeIndex::new((r + 1) * cols + c), EdgeLabel::Real);
            }
            if c + 1 < cols {
                graph.add_edge(NodeIndex::new(r * cols + c), NodeIndex::new(r * cols + c + 1), EdgeLabel::Real);
            }
        }
    }

    graph
}
