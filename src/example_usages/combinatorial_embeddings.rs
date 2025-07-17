use embed_doc_image::embed_doc_image;

use crate::{
    UnGraph, spqr_blocks::outside_structures::SPQRTree, spqr_tree::get_spqr_tree,
    triconnected_blocks::outside_structures::ComponentType,
};

/// Counts the number of all combinatorial embeddings of a given planar graph.
///
/// Based on (https://www.ac.tuwien.ac.at/files/pub/Mutzel99.pdf)
/// and (https://www.sciencedirect.com/science/article/pii/0012365X9390316L)
pub fn count_combinatorial_embeddings(graph: &UnGraph) -> usize {
    let mut embeddings = 1;
    unimplemented!();
    embeddings
}

/// Counts the number of combinatorial embeddings in a biconnected planar graph using the SPQR tree.
///
/// Combinatorial embedding is defined by clockwise (or anticlockwise) order of edges around each vertex.
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
pub fn count_combinatorial_embeddings_biconnected(
    graph: &UnGraph
) -> usize {
    let spqr_tree = get_spqr_tree(graph);
    let mut embeddings = 1;
    let mut p_nodes = 0;

    for component in &spqr_tree.triconnected_components.components {
        match component.component_type {
            Some(ComponentType::P) => {
                let k = component.edges.len();
                // summing, because each P node contains others P nodes in some virt edge
                p_nodes += (1..k).product::<usize>();
            }
            Some(ComponentType::R) => {
                embeddings *= 2;
            }
            _ => continue,
        }
    }

    embeddings *= if p_nodes == 0 { 1 } else { p_nodes };

    embeddings
}

mod tests {
    use crate::EdgeLabel;
    use super::*;

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
        assert_eq!(embeddings, 8); // only one way to embed a bond
    }

}