use hashbrown::HashMap;
use petgraph::visit::{IntoNodeReferences, NodeIndexable};

use crate::{
    UnGraph,
    block_cut::{BlockCutTree, get_block_cut_tree},
    example_usages::static_triconnectivity_bicon::StaticBiconnectedTriconnectivity,
};

/// Implements a static triconnectivity algorithm.
///
/// Using the SPQR-tree and block-cut tree structures, this algorithm after a linear preprocessing answers queries in form `Are vertices a and b in the same triconnected component?` in constant time.
///
/// Prerequisite: input graph is connected.
///
/// ## Reference:
/// - [On-line maintenance of triconnected components with SPQR-trees](https://link.springer.com/article/10.1007/BF01961541)

pub struct StaticTriconnectivity {
    tree: BlockCutTree,

    triconnectivity_blocks: Vec<StaticBiconnectedTriconnectivity>, // for each block in the bct we store it's corresponding triconnectivity query structure
    vertex_numbers_mapping: Vec<HashMap<usize, usize>>, // vertices inside the spqr trees are numbered from 0 to m-1, so here
    // we map the original vertex numbers to the new ones
    parent: Vec<Option<usize>>, // for each vertex in the bct we store it's parent
}

impl StaticTriconnectivity {
    pub fn new(graph: &UnGraph) -> Self {
        let bct = get_block_cut_tree(&graph);

        let mut triconnectivity_blocks = Vec::with_capacity(bct.blocks.len());
        let mut vertex_numbers_mapping = Vec::with_capacity(bct.node_to_id.len());

        for block in bct.blocks.iter() {
            triconnectivity_blocks.push(StaticBiconnectedTriconnectivity::new(&block));

            vertex_numbers_mapping.push(HashMap::new());
            for (i, v) in block.node_references().enumerate() {
                vertex_numbers_mapping
                    .last_mut()
                    .unwrap()
                    .insert(*v.1 as usize, i);
            }
        }

        let mut parent = vec![None; bct.graph.node_count()];
        fn dfs(bct: &BlockCutTree, u: usize, parent: &mut Vec<Option<usize>>) {
            for v in bct.graph.neighbors(bct.graph.from_index(u)) {
                let to = v.index();

                if parent[to].is_none() {
                    parent[to] = Some(u);
                    dfs(bct, to, parent);
                }
            }
        }

        dfs(&bct, 0, &mut parent);

        StaticTriconnectivity {
            tree: bct,
            triconnectivity_blocks,
            vertex_numbers_mapping,
            parent,
        }
    }

    fn check_block(&self, block_id: usize, a: usize, b: usize) -> bool {
        if let Some(a_inside) = self.vertex_numbers_mapping[block_id].get(&a) {
            if let Some(b_inside) = self.vertex_numbers_mapping[block_id].get(&b) {
                return self.triconnectivity_blocks[block_id].query(*a_inside, *b_inside, false);
            }
        }
        false
    }

    pub fn query(&self, a: usize, b: usize, rep: bool) -> bool {
        if a == b {
            return true; // trivial case
        }

        if self.tree.node_to_id[a] < self.tree.block_count {
            // a is fully inside some block
            if self.check_block(self.tree.node_to_id[a], a, b) {
                return true;
            }
        } else if let Some(p) = self.parent[self.tree.node_to_id[a]] {
            // a is a cut vertex, check its parent (a block)
            if self.check_block(p, a, b) {
                return true;
            }
        }

        if !rep {
            return self.query(b, a, true);
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use petgraph::visit::{EdgeRef, IntoNodeReferences};

    use crate::testing::random_graphs::random_graph;

    use super::*;

    struct StaticTriconnectivityBrute {
        cap: Vec<Vec<usize>>,
    }
    impl StaticTriconnectivityBrute {
        pub fn new(graph: &UnGraph) -> Self {
            let n = graph.node_references().count();
            let mut cap = vec![vec![0; n * 2]; n * 2]; // indices from 0 to n-1 are 'ins', rest are 'outs'

            for (u, v) in graph
                .edge_references()
                .map(|e| (e.source().index(), e.target().index()))
            {
                cap[u + n][v] += 1;
                cap[v + n][u] += 1;
            }
            for u in 0..n {
                cap[u][u + n] += 1; // ins to outs
            }

            StaticTriconnectivityBrute { cap }
        }
        pub fn query(&self, a: usize, b: usize) -> bool {
            if a == b {
                return true;
            }

            let mut cap = self.cap.clone();
            let mut vis = vec![false; cap.len()];
            fn dfs(u: usize, t: usize, cap: &mut Vec<Vec<usize>>, vis: &mut Vec<bool>) -> bool {
                vis[u] = true;
                if u == t {
                    return true;
                }
                for v in 0..cap.len() {
                    if !vis[v] && cap[u][v] > 0 {
                        if dfs(v, t, cap, vis) {
                            cap[u][v] -= 1;
                            cap[v][u] += 1;
                            return true;
                        }
                    }
                }
                false
            }
            for _ in 0..3 {
                if !dfs(a + cap.len() / 2, b, &mut cap, &mut vis) {
                    return false;
                }
                vis.fill(false);
            }
            true
        }
    }

    #[test]
    fn test_triconnectivity_light() {
        for i in 0..100 {
            let n = 2 + i / 10;
            let m: usize = 1 + i;

            let in_graph = random_graph(n, m, i);

            let fast_triconnectivity: StaticTriconnectivity = StaticTriconnectivity::new(&in_graph);
            let slow_triconnectivity = StaticTriconnectivityBrute::new(&in_graph);

            for u in 0..n {
                for v in 0..n {
                    assert_eq!(
                        fast_triconnectivity.query(u, v, false),
                        slow_triconnectivity.query(u, v),
                    );
                }
            }
        }
    }

    #[cfg(all(test, not(debug_assertions)))]
    #[test]
    fn test_triconnectivity_exhaustive() {
        use crate::testing::graph_enumerator::GraphEnumeratorState;
        use petgraph::graph::NodeIndex;
        use petgraph::prelude::Dfs;

        fn is_connected(graph: &UnGraph) -> bool {
            let mut dfs = Dfs::new(graph, NodeIndex::new(0));
            let mut visited = 0;
            while let Some(_) = dfs.next(graph) {
                visited += 1;
            }
            visited == graph.node_count()
        }

        for n in 2..=7 {
            let mut enumerator = GraphEnumeratorState {
                n,
                mask: 0,
                last_mask: (1 << (n * (n - 1) / 2)),
            };

            while let Some(in_graph) = enumerator.next() {
                let n = in_graph.node_references().count();
                if !is_connected(&in_graph) {
                    continue; // skip disconnected graphs
                }

                let fast_triconnectivity: StaticTriconnectivity =
                    StaticTriconnectivity::new(&in_graph);
                let slow_triconnectivity = StaticTriconnectivityBrute::new(&in_graph);

                for u in 0..n {
                    for v in 0..n {
                        assert_eq!(
                            fast_triconnectivity.query(u, v, false),
                            slow_triconnectivity.query(u, v)
                        );
                    }
                }
            }
        }
    }

    #[cfg(all(test, not(debug_assertions)))]
    #[test]
    fn test_triconnectivity() {
        for i in 0..1000 {
            let n = 2 + i / 10;
            let m: usize = 1 + i;

            let in_graph = random_graph(n, m, i);

            let fast_triconnectivity: StaticTriconnectivity = StaticTriconnectivity::new(&in_graph);
            let slow_triconnectivity = StaticTriconnectivityBrute::new(&in_graph);

            for u in 0..n {
                for v in 0..n {
                    assert_eq!(
                        fast_triconnectivity.query(u, v, false),
                        slow_triconnectivity.query(u, v),
                    );
                }
            }
        }
    }
}
