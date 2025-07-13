use crate::{
    UnGraph, spqr_blocks::outside_structures::SPQRTree, triconnected::get_triconnected_components,
};

pub fn get_spqr_tree(graph: &UnGraph) -> SPQRTree {
    let triconnected_components = get_triconnected_components(graph);

    let mut spqr_tree = SPQRTree::new(&triconnected_components);

    // now we just add edges between components
    let mut edge_to_component = vec![0; triconnected_components.edges.len()];
    for (i, component) in triconnected_components.components.iter().enumerate() {
        for &eid in &component.edges {
            edge_to_component[eid] = i;
        }
    }

    for (i, component) in triconnected_components.components.iter().enumerate() {
        for &eid in &component.edges {
            if edge_to_component[eid] == i {
                continue;
            }

            spqr_tree.add_edge(i, edge_to_component[eid]);
        }
    }

    spqr_tree
}

#[cfg(test)]
mod tests {
    use std::mem;

    use petgraph::visit::{EdgeRef, IntoNodeReferences};

    use crate::{
        block_cut::get_block_cut_tree,
        testing::{
            graph_enumerator::GraphEnumeratorState, random_graphs::random_biconnected_graph,
        },
    };

    use super::*;

    fn same_graphs(og_graph: &UnGraph, spqr_tree: &SPQRTree) -> bool {
        let mut edge_counts = vec![0; spqr_tree.triconnected_components.edges.len()];

        let mut vis = vec![false; spqr_tree.triconnected_components.components.len()];
        fn dfs(
            spqr_tree: &SPQRTree,
            component_id: usize,
            edge_counts: &mut Vec<usize>,
            vis: &mut Vec<bool>,
        ) {
            vis[component_id] = true;

            for &eid in &spqr_tree.triconnected_components.components[component_id].edges {
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
                let (mut u, mut v) = spqr_tree.triconnected_components.edges[eid];
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
