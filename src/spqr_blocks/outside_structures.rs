use crate::triconnected_blocks::outside_structures::TriconnectedComponents;

#[derive(Debug, Clone)]
pub struct SPQRTree {
    pub triconnected_components: TriconnectedComponents,
    pub adj: Vec<Vec<usize>>,
}

impl SPQRTree {
    pub fn new(triconnected_components: TriconnectedComponents) -> Self {
        let n = triconnected_components.components.len();
        let adj = vec![Vec::new(); n];
        SPQRTree {
            triconnected_components,
            adj,
        }
    }
    pub(crate) fn add_edge(&mut self, u: usize, v: usize) {
        self.adj[u].push(v);
        self.adj[v].push(u);
    }
}
