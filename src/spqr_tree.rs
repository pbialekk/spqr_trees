use embed_doc_image::embed_doc_image;

use crate::{
    UnGraph,
    spqr_blocks::outside_structures::{RootedSPQRTree, SPQRTree},
    triconnected::get_triconnected_components,
};

/// ## Overwiew
/// Given a biconnected graph `G`, this function returns its SPQR tree which is computed in linear time.
///
/// For more information, refer to the triconnected.rs module documentation.
///
/// ## Example (visualized using .dot file generated with visualize.rs from spqr_blocks)
/// ![SPQR_Full][spqr_full]
#[embed_doc_image("spqr_full", "assets/spqr_tree.svg")]
pub fn get_spqr_tree(graph: &UnGraph) -> SPQRTree {
    let triconnected_components = get_triconnected_components(graph);

    let mut spqr_tree = SPQRTree::new(&triconnected_components);

    // now we just add edges between components
    let mut edge_to_component = vec![0; triconnected_components.edges.len()];
    for (i, component) in triconnected_components.comp.iter().enumerate() {
        for &eid in &component.edges {
            edge_to_component[eid] = i;
        }
    }

    for (i, component) in triconnected_components.comp.iter().enumerate() {
        for &eid in &component.edges {
            if edge_to_component[eid] == i {
                continue;
            }

            spqr_tree.add_edge(i, edge_to_component[eid]);
        }
    }

    spqr_tree
}

/// ## Overwiew
/// Given a biconnected graph `G`, this function returns its rooted SPQR tree at the first component.
///
/// After rooting the tree, `adj[u]` doesn't contain the parent component of `u` in the SPQR tree.
pub fn get_rooted_spqr_tree(graph: &UnGraph) -> RootedSPQRTree {
    let unrooted_spqr = get_spqr_tree(graph);
    let mut rooted_spqr = RootedSPQRTree::new(&unrooted_spqr);

    let mut mark = vec![false; rooted_spqr.blocks.edges.len()];
    fn root_tree(tree: &mut RootedSPQRTree, u: usize, mark: &mut Vec<bool>) {
        for &eid in tree.blocks.comp[u].edges.iter() {
            if mark[eid] {
                tree.ref_edge[u] = Some(eid);
            }

            let (a, b) = tree.blocks.edges[eid];

            for turn in [a, b] {
                if tree.alloc_node[turn] == usize::MAX {
                    tree.alloc_node[turn] = u;
                }
            }

            mark[eid] = true;
        }

        // remove edge to parent
        if let Some(parent) = tree.par_v[u] {
            tree.adj[u].retain(|&x| x != parent);
        }

        let neighbors = tree.adj[u].clone();

        for &to in neighbors.iter() {
            tree.par_v[to] = Some(u);
            root_tree(tree, to, mark);
        }
    }

    if rooted_spqr.blocks.comp.len() > 0 {
        root_tree(&mut rooted_spqr, 0, &mut mark);
    }

    rooted_spqr
}

#[cfg(test)]
mod tests {
    use std::mem;

    use petgraph::visit::{EdgeRef, IntoNodeReferences};

    use crate::testing::random_graphs::random_biconnected_graph;

    use super::*;

    fn same_graphs(og_graph: &UnGraph, spqr_tree: &SPQRTree) -> bool {
        let mut edge_counts = vec![0; spqr_tree.blocks.edges.len()];

        let mut vis = vec![false; spqr_tree.blocks.comp.len()];
        fn dfs(
            spqr_tree: &SPQRTree,
            component_id: usize,
            edge_counts: &mut Vec<usize>,
            vis: &mut Vec<bool>,
        ) {
            vis[component_id] = true;

            for &eid in &spqr_tree.blocks.comp[component_id].edges {
                edge_counts[eid] += 1;
            }

            for &neigh in &spqr_tree.adj[component_id] {
                if vis[neigh] {
                    continue;
                }
                dfs(spqr_tree, neigh, edge_counts, vis);
            }
        }

        if og_graph.node_references().count() == 2 && og_graph.edge_references().count() < 3 {
            return true;
        }

        dfs(spqr_tree, 0, &mut edge_counts, &mut vis);
        assert!(vis.iter().all(|&x| x));

        let mut spq_edges = vec![];
        for (eid, count) in edge_counts.iter().enumerate() {
            if *count == 1 {
                let (mut u, mut v) = spqr_tree.blocks.edges[eid];
                if u > v {
                    mem::swap(&mut u, &mut v);
                }
                spq_edges.push((u, v));
            } else {
                assert!(*count <= 2); // either a vedge deleted long ago due to merging P and S nodes, or a vedge
            }
        }

        spq_edges.sort();

        let mut edges_in = vec![];
        for edge in og_graph.edge_references() {
            let mut u = edge.source().index();
            let mut v = edge.target().index();
            if u > v {
                mem::swap(&mut u, &mut v);
            }
            edges_in.push((u, v));
        }
        edges_in.sort();

        spq_edges == edges_in
    }

    #[cfg(all(test, not(debug_assertions)))]
    #[test]
    fn test_spqr_tree() {
        for i in 0..1000 {
            println!("test_spqr_tree_light() it: {}", i);

            let n = 2 + i / 10;
            let m: usize = 1 + i;

            let in_graph = random_biconnected_graph(n, m, i);
            let spqr_tree = get_spqr_tree(&in_graph);
            assert!(same_graphs(&in_graph, &spqr_tree));
        }
    }

    #[test]
    fn test_spqr_tree_light() {
        for i in 0..100 {
            println!("test_spqr_tree_light() it: {}", i);

            let n = 2 + i / 10;
            let m: usize = 1 + i;

            let in_graph = random_biconnected_graph(n, m, i);
            let spqr_tree = get_spqr_tree(&in_graph);
            assert!(same_graphs(&in_graph, &spqr_tree));
        }
    }

    #[cfg(all(test, not(debug_assertions)))]
    #[test]
    fn test_spqr_tree_exhaustive() {
        use crate::{
            block_cut::get_block_cut_tree, testing::graph_enumerator::GraphEnumeratorState,
        };

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

                let spqr_tree = get_spqr_tree(&in_graph);
                assert!(same_graphs(&in_graph, &spqr_tree));
            }
        }
    }
}
