use crate::{
    UnGraph, block_cut::get_block_cut_tree, spqr_tree::get_spqr_tree,
    triconnected_blocks::outside_structures::ComponentType,
};
use embed_doc_image::embed_doc_image;
use hashbrown::HashMap;
use petgraph::graph::NodeIndex;

/// Counts the number of all combinatorial embeddings of a given graph in O(V + E).
///
/// Combinatorial embedding is defined by clockwise (or counterclockwise) order of edges around each vertex.
///
/// Based on (https://www.sciencedirect.com/science/article/pii/0012365X9390316L)
///
/// Explanation (symbols aren't aligned with the paper for readability):
/// - `block_emb` - number of combinatorial embeddings for each block in the block-cut tree
/// - `deg_in_bc` - degree of the cut vertex in the block-cut tree
/// - `edges_adj_v` - number of edges (from original graph) adjacent to the cut vertex (for each block)
///
/// # Idea:
/// 1. Take into account embeddings of biconnected components.
/// 2. Choose first edges around cut vertices for each block.
/// 3. Account for permutations of all edges around cut vertices that do not contain interlacing.
///
/// This algorithm can give you an idea how to go through all the embeddings and choose appropriate
/// based on given conditions.
pub fn count_combinatorial_embeddings(graph: &UnGraph) -> usize {
    let bc_tree = get_block_cut_tree(graph);

    if bc_tree.block_count == 1 {
        return count_combinatorial_embeddings_biconnected(graph);
    }

    // we know our nodes have distinct labels
    let mut label_to_index = HashMap::<u32, NodeIndex>::new();
    for node in graph.node_indices() {
        label_to_index.insert(*graph.node_weight(node).unwrap(), node);
    }

    // init with 1 because we are multiplying
    let mut block_emb = vec![1; bc_tree.block_count];
    let mut deg_in_bc = vec![1; bc_tree.cut_count];
    let mut deg_in_og = vec![1; bc_tree.cut_count];
    let mut edges_adj_v = vec![vec![1; bc_tree.block_count]; bc_tree.cut_count];

    let mut cut_labels = HashMap::<u32, usize>::new();
    for i in 0..bc_tree.cut_count {
        let idx = bc_tree.block_count + i;
        deg_in_bc[i] = bc_tree.graph.neighbors(NodeIndex::new(idx)).count();
        let label = *bc_tree.graph.node_weight(NodeIndex::new(idx)).unwrap();
        cut_labels.insert(label, i);
        deg_in_og[i] = graph.neighbors(label_to_index[&label]).count();
    }

    for (i, block) in bc_tree.blocks.iter().enumerate() {
        block_emb[i] = count_combinatorial_embeddings_biconnected(block);
        for v in block.node_indices() {
            if cut_labels.contains_key(block.node_weight(v).unwrap()) {
                let cut_idx = cut_labels[block.node_weight(v).unwrap()];
                edges_adj_v[cut_idx][i] = block.neighbors(v).count();
            }
        }
    }

    let mut embeddings = 1;
    // this part accounts for biconnected components embeddings
    for i in 0..bc_tree.block_count {
        embeddings *= block_emb[i];
        // this part accounts for choosing first edges
        for j in 0..bc_tree.cut_count {
            embeddings *= edges_adj_v[j][i];
        }
    }
    // this part accounts for permutation of edges of biconnected component around cut vertex
    // but restricted to not contain interlacing (1 1 1 2 2 1 1 2 2 2 - numbers indicate component of edge)
    for i in 0..bc_tree.cut_count {
        for j in 1..deg_in_bc[i] - 1 {
            embeddings *= deg_in_og[i] - j;
        }
    }

    embeddings
}

/// Counts the number of combinatorial embeddings in a biconnected graph using the SPQR tree.
///
/// Combinatorial embedding is defined by clockwise (or counterclockwise) order of edges around each vertex.
///
/// The idea is simple, we loop over all components of SPQR tree of given graph:
/// - **S node (cycle)** - has only 1 embedding
/// - **P node (bond)** - (k-1)! embeddings, where k is the number of edges in the bond (NOTE: not k!)
/// - **R node (triconnected component)** - has 2 embeddings (we count also mirror reflection)
///
/// You can try it yourself on the envelope on 8 vertices.
///
/// SPQR tree of this graph:
///
/// ![SQPR_Envelope][spqr_envelope]
#[embed_doc_image("spqr_envelope", "assets/spqr_tree_envelope.svg")]
pub fn count_combinatorial_embeddings_biconnected(graph: &UnGraph) -> usize {
    if graph.node_count() <= 1 {
        return 1;
    }

    let spqr_tree = get_spqr_tree(graph);
    let mut embeddings = 1;

    for component in &spqr_tree.blocks.comp {
        match component.comp_type {
            ComponentType::P => {
                let k = component.edges.len();
                embeddings *= (1..k).product::<usize>();
            }
            ComponentType::R => {
                embeddings *= 2;
            }
            _ => continue,
        }
    }

    embeddings
}

mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::EdgeLabel;

    #[test]
    fn test_count_combinatorial_embeddings_biconnected_single_vertex() {
        let mut graph = UnGraph::new_undirected();
        graph.add_node(0);
        let embeddings = count_combinatorial_embeddings_biconnected(&graph);
        assert_eq!(embeddings, 1);
    }

    #[test]
    fn test_count_combinatorial_embeddings_biconnected_single_edge() {
        let mut graph = UnGraph::new_undirected();
        graph.add_node(0);
        graph.add_node(1);
        graph.add_edge(0.into(), 1.into(), EdgeLabel::Real);
        let embeddings = count_combinatorial_embeddings_biconnected(&graph);
        assert_eq!(embeddings, 1);
    }

    #[test]
    fn test_count_combinatorial_embeddings_biconnected_triconnected() {
        let mut graph = UnGraph::new_undirected();
        graph.add_node(0);
        graph.add_node(1);
        graph.add_node(2);
        graph.add_edge(0.into(), 1.into(), EdgeLabel::Real);
        graph.add_edge(1.into(), 2.into(), EdgeLabel::Real);
        graph.add_edge(2.into(), 0.into(), EdgeLabel::Real);

        let embeddings = count_combinatorial_embeddings_biconnected(&graph);
        assert_eq!(embeddings, 1);
    }

    #[test]
    fn test_count_combinatorial_embeddings_biconnected_bond() {
        let mut graph = UnGraph::new_undirected();
        graph.add_node(0);
        graph.add_node(1);
        graph.add_edge(0.into(), 1.into(), EdgeLabel::Real);
        graph.add_edge(0.into(), 1.into(), EdgeLabel::Real);
        graph.add_edge(0.into(), 1.into(), EdgeLabel::Real);
        graph.add_edge(0.into(), 1.into(), EdgeLabel::Real);
        graph.add_edge(0.into(), 1.into(), EdgeLabel::Real);

        let embeddings = count_combinatorial_embeddings_biconnected(&graph);
        assert_eq!(embeddings, 24);
    }

    #[test]
    fn test_count_combinatorial_embeddings_envelope() {
        let mut graph = UnGraph::new_undirected();
        graph.add_node(0);
        graph.add_node(1);
        graph.add_node(2);
        graph.add_node(3);
        graph.add_node(4);
        graph.add_edge(0.into(), 1.into(), EdgeLabel::Real);
        graph.add_edge(1.into(), 2.into(), EdgeLabel::Real);
        graph.add_edge(2.into(), 3.into(), EdgeLabel::Real);
        graph.add_edge(3.into(), 0.into(), EdgeLabel::Real);
        graph.add_edge(0.into(), 4.into(), EdgeLabel::Real);
        graph.add_edge(3.into(), 4.into(), EdgeLabel::Real);

        let embeddings = count_combinatorial_embeddings_biconnected(&graph);
        assert_eq!(embeddings, 2);
    }

    #[test]
    fn test_count_combinatorial_embeddings_diamond() {
        let mut graph = UnGraph::new_undirected();
        graph.add_node(0);
        graph.add_node(1);
        graph.add_node(2);
        graph.add_node(3);
        graph.add_node(4);
        graph.add_node(5);
        graph.add_node(6);
        graph.add_node(7);
        graph.add_edge(0.into(), 1.into(), EdgeLabel::Real);
        graph.add_edge(1.into(), 2.into(), EdgeLabel::Real);
        graph.add_edge(2.into(), 3.into(), EdgeLabel::Real);
        graph.add_edge(3.into(), 0.into(), EdgeLabel::Real);
        graph.add_edge(0.into(), 4.into(), EdgeLabel::Real);
        graph.add_edge(3.into(), 4.into(), EdgeLabel::Real);
        graph.add_edge(0.into(), 5.into(), EdgeLabel::Real);
        graph.add_edge(1.into(), 5.into(), EdgeLabel::Real);
        graph.add_edge(1.into(), 6.into(), EdgeLabel::Real);
        graph.add_edge(2.into(), 6.into(), EdgeLabel::Real);
        graph.add_edge(2.into(), 7.into(), EdgeLabel::Real);
        graph.add_edge(3.into(), 7.into(), EdgeLabel::Real);

        let embeddings = count_combinatorial_embeddings_biconnected(&graph);
        assert_eq!(embeddings, 16);
    }

    #[test]
    fn test_count_combinatorial_embeddings_single_edge() {
        let mut graph = UnGraph::new_undirected();
        graph.add_node(0);
        graph.add_node(1);
        graph.add_edge(0.into(), 1.into(), EdgeLabel::Real);

        let embeddings = count_combinatorial_embeddings(&graph);
        assert_eq!(embeddings, 1);
    }

    #[test]
    fn test_count_combinatorial_embeddings_path() {
        let mut graph = UnGraph::new_undirected();
        graph.add_node(0);
        graph.add_node(1);
        graph.add_node(2);
        graph.add_node(3);
        graph.add_edge(0.into(), 1.into(), EdgeLabel::Real);
        graph.add_edge(1.into(), 2.into(), EdgeLabel::Real);
        graph.add_edge(2.into(), 3.into(), EdgeLabel::Real);

        let embeddings = count_combinatorial_embeddings(&graph);
        assert_eq!(embeddings, 1);
    }

    #[test]
    fn test_count_combinatorial_embeddings_cycle() {
        let mut graph = UnGraph::new_undirected();
        graph.add_node(0);
        graph.add_node(1);
        graph.add_node(2);
        graph.add_node(3);
        graph.add_edge(0.into(), 1.into(), EdgeLabel::Real);
        graph.add_edge(1.into(), 2.into(), EdgeLabel::Real);
        graph.add_edge(2.into(), 3.into(), EdgeLabel::Real);
        graph.add_edge(3.into(), 0.into(), EdgeLabel::Real);

        let embeddings = count_combinatorial_embeddings(&graph);
        assert_eq!(embeddings, 1);
    }

    #[test]
    fn test_count_combinatorial_embeddings_paper() {
        // graph from the paper
        let mut graph = UnGraph::new_undirected();
        graph.add_node(0);
        graph.add_node(1);
        graph.add_node(2);
        graph.add_node(3);
        graph.add_node(4);
        graph.add_node(5);
        graph.add_node(6);
        graph.add_node(7);
        graph.add_node(8);
        graph.add_edge(0.into(), 1.into(), EdgeLabel::Real); // c
        graph.add_edge(0.into(), 2.into(), EdgeLabel::Real); // a
        graph.add_edge(0.into(), 3.into(), EdgeLabel::Real); // b
        graph.add_edge(2.into(), 3.into(), EdgeLabel::Real);
        graph.add_edge(0.into(), 4.into(), EdgeLabel::Real); // h
        graph.add_edge(0.into(), 5.into(), EdgeLabel::Real); // g
        graph.add_edge(4.into(), 5.into(), EdgeLabel::Real);
        graph.add_edge(0.into(), 6.into(), EdgeLabel::Real); // e
        graph.add_edge(0.into(), 7.into(), EdgeLabel::Real); // f
        graph.add_edge(6.into(), 7.into(), EdgeLabel::Real);
        graph.add_edge(0.into(), 8.into(), EdgeLabel::Real); // d
        graph.add_edge(6.into(), 8.into(), EdgeLabel::Real);

        let embeddings = count_combinatorial_embeddings(&graph);
        assert_eq!(embeddings, 1008);
    }

    #[test]
    fn test_count_combinatorial_embeddings_random_tree() {
        // for tree this is simple: for all v, product((deg(v)-1)!)
        fn brute_embeddings_tree(tree: &UnGraph) -> usize {
            if tree.node_count() <= 1 {
                return 1;
            }
            let mut embeddings = 1;
            for node in tree.node_indices() {
                embeddings *= (1..tree.neighbors(node).count()).product::<usize>();
            }
            embeddings
        }
        for i in 1..50 {
            let graph = crate::testing::random_graphs::random_tree(i, 42);
            let embeddings = count_combinatorial_embeddings(&graph);
            let brute = brute_embeddings_tree(&graph);
            assert_eq!(embeddings, brute);
        }
    }
}
