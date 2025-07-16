use hashbrown::HashMap;
use petgraph::visit::IntoNodeReferences;

use crate::{
    UnGraph, spqr_blocks::outside_structures::SPQRTree, spqr_tree::get_spqr_tree,
    triconnected_blocks::outside_structures::ComponentType,
};

/// Implements a static triconnectivity algorithm.
///
/// Using the SPQR-tree structure, this algorithm after a linear preprocessing answers queries in form `Are vertices a and b in the same triconnected component?` in constant time.
///
/// Prerequisite: input graph is biconnected
///
/// ## Reference:
/// - [On-line maintenance of triconnected components with SPQR-trees](https://link.springer.com/article/10.1007/BF01961541)
pub struct StaticTriconnectivity {
    tree: SPQRTree,

    reference_edge: Vec<Option<usize>>,
    allocation_node: Vec<usize>,
    s_links: Vec<HashMap<usize, (Option<usize>, Option<usize>)>>,
}

impl StaticTriconnectivity {
    pub fn new(graph: &UnGraph) -> Self {
        let tree = get_spqr_tree(&graph);

        let mut reference_edge = vec![None; tree.adj.len()];
        let mut allocation_node = vec![None; graph.node_references().count()];
        let mut s_links = vec![HashMap::new(); tree.adj.len()];

        // We have to make our tree rooted in order to efficiently answer queries.

        let mut mark = vec![false; tree.triconnected_components.edges.len()];
        fn root_tree(
            tree: &SPQRTree,
            reference_edge: &mut Vec<Option<usize>>,
            allocation_node: &mut Vec<Option<usize>>,
            u: usize,
            parent: Option<usize>,
            mark: &mut Vec<bool>,
            s_links: &mut Vec<HashMap<usize, (Option<usize>, Option<usize>)>>,
        ) {
            for &eid in tree.triconnected_components.components[u].edges.iter() {
                if mark[eid] {
                    reference_edge[u] = Some(eid);
                }

                let (a, b) = tree.triconnected_components.edges[eid];

                for turn in [a, b] {
                    if allocation_node[turn].is_none() {
                        allocation_node[turn] = Some(u);
                    }

                    if tree.triconnected_components.components[u].component_type
                        == Some(ComponentType::S)
                        && !mark[eid]
                        && !tree.triconnected_components.is_real_edge[eid]
                    {
                        let entry = s_links[u].entry(turn).or_insert((None, None));
                        if entry.0.is_none() {
                            entry.0 = Some(eid);
                        } else {
                            entry.1 = Some(eid);
                        }
                    }
                }

                mark[eid] = true;
            }

            for &to in tree.adj[u].iter() {
                if Some(to) == parent {
                    continue;
                }
                root_tree(
                    tree,
                    reference_edge,
                    allocation_node,
                    to,
                    Some(u),
                    mark,
                    s_links,
                );
            }
        }

        if tree.triconnected_components.components.len() > 0 {
            root_tree(
                &tree,
                &mut reference_edge,
                &mut allocation_node,
                0,
                None,
                &mut mark,
                &mut s_links,
            );

            StaticTriconnectivity {
                tree,
                reference_edge,
                allocation_node: allocation_node.into_iter().map(|x| x.unwrap()).collect(),
                s_links,
            }
        } else {
            StaticTriconnectivity {
                tree,
                reference_edge: vec![],
                allocation_node: vec![],
                s_links: vec![],
            }
        }
    }

    fn are_poles(&self, a: usize, b: usize, link: Option<usize>) -> bool {
        if let Some(link) = link {
            let (s, t) = self.tree.triconnected_components.edges[link];
            if a == b {
                return s == b || t == b;
            } else {
                return (s, t) == (a, b) || (s, t) == (b, a);
            }
        }
        false
    }

    /// Returns true iff the vertices `a` and `b` are in the same triconnected component.
    pub fn query(&self, a: usize, b: usize, rep: bool) -> bool {
        if a == b {
            return true;
        }

        if self.tree.triconnected_components.components.len() == 0 {
            return false;
        }

        let proper_a = self.allocation_node[a];
        let proper_b = self.allocation_node[b];

        let proper_a_type = self.tree.triconnected_components.components[proper_a].component_type;

        if proper_a == proper_b
            && (proper_a_type == Some(ComponentType::R) || proper_a_type == Some(ComponentType::P))
        {
            return true;
        }
        if proper_a_type == Some(ComponentType::R) {
            let ref_edge = self.reference_edge[proper_a];
            if let Some(ref_edge) = ref_edge {
                let (s, t) = self.tree.triconnected_components.edges[ref_edge];
                if s == b || t == b {
                    return true;
                }
            }
        }
        if proper_a_type == Some(ComponentType::S) {
            if let Some(&(link_1, link_2)) = self.s_links[proper_a].get(&a) {
                if self.are_poles(a, b, link_1) || self.are_poles(a, b, link_2) {
                    return true;
                }
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
    use petgraph::visit::EdgeRef;

    use crate::{
        block_cut::get_block_cut_tree,
        testing::{
            graph_enumerator::GraphEnumeratorState, random_graphs::random_biconnected_graph,
        },
    };

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

            let in_graph = random_biconnected_graph(n, m, i);

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
        for n in 2..=7 {
            let mut enumerator = GraphEnumeratorState {
                n,
                mask: 0,
                last_mask: (1 << (n * (n - 1) / 2)),
            };

            while let Some(in_graph) = enumerator.next() {
                let bct = get_block_cut_tree(&in_graph);
                if bct.cut_count > 0 || bct.block_count == 0 {
                    continue; // not biconnected
                }

                let in_graph = bct.blocks[0].clone();
                let n = in_graph.node_references().count();

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

            let in_graph = random_biconnected_graph(n, m, i);

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
