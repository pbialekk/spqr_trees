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

/// Represents a rooted SPQR tree. In addition to the SPQR tree structure,
/// it contains additional information for rooting the tree:
/// - `allocation_node[u]`: Lowest component that contains a vertex 'u'.
/// - `reference_edge[v]`: For a component `v`, it defines the vedge that is common between `v` and `parent(v)` in the SPQR tree.
/// - `parent_node[v]`: Parent component of `v` in the SPQR tree
#[derive(Debug, Clone)]
pub struct RootedSPQRTree {
    pub triconnected_components: TriconnectedComponents,
    pub adj: Vec<Vec<usize>>,

    pub allocation_node: Vec<usize>,
    pub reference_edge: Vec<Option<usize>>,
    pub parent_node: Vec<Option<usize>>,
}

impl RootedSPQRTree {
    pub fn new(spqr_tree: &SPQRTree) -> Self {
        let n_comps = spqr_tree.adj.len();
        let n_verts = spqr_tree
            .triconnected_components
            .edges
            .iter()
            .map(|(a, b)| a.max(b))
            .max()
            .unwrap()
            + 1;
        RootedSPQRTree {
            triconnected_components: spqr_tree.triconnected_components.clone(),
            adj: spqr_tree.adj.clone(),
            allocation_node: vec![usize::MAX; n_verts],
            reference_edge: vec![None; n_comps],
            parent_node: vec![None; n_comps],
        }
    }
}
