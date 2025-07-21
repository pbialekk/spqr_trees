use petgraph::visit::NodeIndexable;

use crate::{EdgeLabel, UnGraph};

#[allow(dead_code)]
pub(crate) struct GraphEnumeratorState {
    pub n: usize,
    pub mask: usize,
    pub last_mask: usize,
}

// Example usage to avoid dead_code warning
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_enumerator_state_construction() {
        let mut enumerator = GraphEnumeratorState {
            n: 3,
            mask: 0,
            last_mask: 7,
        };
        // Just call next to ensure it's used
        let _ = enumerator.next();
    }
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
