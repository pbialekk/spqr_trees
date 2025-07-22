use crate::EdgeLabel;
use crate::UnGraph;
use crate::block_cut::get_block_cut_tree;
use petgraph::visit::NodeIndexable;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

#[allow(dead_code)]
pub fn random_graph(n: usize, m: usize, seed: usize) -> UnGraph {
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

    graph
}

#[allow(dead_code)]
pub fn random_biconnected_graph(n: usize, m: usize, seed: usize) -> UnGraph {
    let graph = random_graph(n, m, seed);

    let bct = get_block_cut_tree(&graph);

    bct.blocks[0].clone()
}
