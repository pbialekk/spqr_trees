use petgraph::visit::NodeIndexable;

use crate::{EdgeLabel, UnGraph};

/// Enumerates all simple graphs on `n` vertices by iterating over edge subsets.
///
/// Each bit in `mask` corresponds to one of the `n*(n-1)/2` possible edges.
#[allow(dead_code)]
pub struct GraphEnumeratorState {
    /// Number of vertices.
    pub n: usize,
    /// Current bitmask of edges.
    pub mask: usize,
    /// One past the last valid bitmask.
    pub last_mask: usize,
}

impl Iterator for GraphEnumeratorState {
    type Item = UnGraph;

    fn next(&mut self) -> Option<Self::Item> {
        if self.mask == self.last_mask {
            return None;
        }

        let mut graph = UnGraph::new_undirected();
        for i in 0..self.n {
            graph.add_node(i.try_into().unwrap());
        }

        let mut check = 0;
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                if self.mask & (1 << check) != 0 {
                    graph.add_edge(graph.from_index(i), graph.from_index(j), EdgeLabel::Real);
                }
                check += 1;
            }
        }

        self.mask = self.mask.wrapping_add(1);
        Some(graph)
    }
}
