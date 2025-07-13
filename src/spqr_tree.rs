use crate::{
    UnGraph, spqr_blocks::outside_structures::SPQRTree, triconnected::get_triconnected_components,
};

pub fn get_spqr_tree(graph: &UnGraph) -> SPQRTree {
    let tricon = get_triconnected_components(graph);

    let mut spqr_tree = SPQRTree::new(tricon.clone());

    // now we just add edges between components
    let mut edge_to_component = vec![0; tricon.edges.len()];
    for (i, component) in tricon.components.iter().enumerate() {
        for &eid in &component.edges {
            edge_to_component[eid] = i;
        }
    }

    for (i, component) in tricon.components.iter().enumerate() {
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

    use petgraph::visit::{EdgeRef, IntoNodeReferences, NodeIndexable};

    use crate::{EdgeLabel, block_cut::get_block_cut_tree};

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

    fn random_biconnected_graph(n: usize, m: usize, seed: usize) -> UnGraph {
        use rand::Rng;
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let mut rng = StdRng::seed_from_u64(seed as u64);
        let mut graph = UnGraph::new_undirected();

        for i in 0..n {
            graph.add_node(i.try_into().unwrap());
            if i > 0 {
                let j = rng.random_range(0..i);
                graph.add_edge(graph.from_index(i), graph.from_index(j), EdgeLabel::Real);
            }
        }

        for _ in n - 1..m {
            let s = rng.random_range(0..n);
            let t = rng.random_range(0..n);
            graph.add_edge(graph.from_index(s), graph.from_index(t), EdgeLabel::Real);
        }

        let bct = get_block_cut_tree(&graph);

        bct.blocks[0].clone()
    }

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
}
