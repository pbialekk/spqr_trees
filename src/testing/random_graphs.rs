use crate::EdgeLabel;
use crate::UnGraph;
use crate::block_cut::get_block_cut_tree;
use petgraph::visit::NodeIndexable;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

/// This function generates a random undirected connected graph.
/// It allows multiple edges and self-loops.
/// Based on spanning tree.
pub(crate) fn random_connected_graph(n: usize, m: usize, seed: usize) -> UnGraph {
    let mut rng = StdRng::seed_from_u64(seed as u64);
    let mut graph = UnGraph::new_undirected();

    for i in 0..n {
        graph.add_node(i.try_into().unwrap());
        if i > 0 {
            let j = rng.random_range(0..i);
            graph.add_edge(graph.from_index(i), graph.from_index(j), EdgeLabel::Real);
        }
    }

    let mut num_edges = n - 1;

    while num_edges < m {
        let s = rng.random_range(0..n);
        let t = rng.random_range(0..n);
        if s == t  {
            continue; // skip self-loops
        }
        graph.add_edge(graph.from_index(s), graph.from_index(t), EdgeLabel::Real);
        num_edges += 1;
    }

    graph
}

/// Generates a random tree.
pub(crate) fn random_tree(n: usize, seed: usize) -> UnGraph {
    let mut rng = StdRng::seed_from_u64(seed as u64);
    let mut graph = UnGraph::new_undirected();

    for i in 0..n {
        graph.add_node(i.try_into().unwrap());
        if i > 0 {
            let j = rng.random_range(0..i);
            graph.add_edge(graph.from_index(i), graph.from_index(j), EdgeLabel::Real);
        }
    }

    graph
} 

/// Generates a random biconnected graph.
/// Takes first biconnected component of BC Tree of a random graph.
pub(crate) fn random_biconnected_graph(n: usize, m: usize, seed: usize) -> UnGraph {
    let graph = random_connected_graph(n, m, seed);

    let bct = get_block_cut_tree(&graph);

    bct.blocks[0].clone()
}
