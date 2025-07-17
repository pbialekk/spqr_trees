use crate::triconnected_blocks::outside_structures::TriconnectedComponents;

/// Represents the SPQR tree structure built from triconnected components.
///
/// - Vertices are numbered from `0` to `k-1`, where `k` is the number of triconnected components.
/// - `adj[u]` contains the indices of components adjacent to component `u` in the SPQR tree.
#[derive(Debug, Clone)]
pub struct SPQRTree {
    pub triconnected_components: TriconnectedComponents,
    pub adj: Vec<Vec<usize>>,
}

impl SPQRTree {
    pub fn new(triconnected_components: &TriconnectedComponents) -> Self {
        let n = triconnected_components.components.len();
        let adj = vec![Vec::new(); n];
        SPQRTree {
            triconnected_components: triconnected_components.clone(),
            adj,
        }
    }
    pub(crate) fn add_edge(&mut self, u: usize, v: usize) {
        self.adj[u].push(v);
        self.adj[v].push(u);
    }
}
