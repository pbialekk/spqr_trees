use petgraph::visit::EdgeRef;

use crate::{
    UnGraph,
    embedding_blocks::{
        acceptable_adj::make_adjacency_lists_acceptable,
        embed::embed_graph,
        kuratowski::get_counterexample,
        lr::dfs2,
        orient::dfs1,
        structures::{GraphInternal, LrOrientation},
    },
    types::DiGraph,
};

/// Implements the LR planarity testing algorithm. Assumes that the input graph is connected.
///
/// Returns a tuple where the first element is a boolean indicating whether the graph is planar, and the second element is either a planar embedding of the graph of it's corresponding kuratowski subgraph if the graph is not planar.
///
/// Reference:
/// [The Left-Right Planarity Test](https://acm.math.spbu.ru/~sk1/download/papers/planar//brandes2010-planarity.pdf)
pub fn is_planar(graph: &UnGraph, with_counterexample: bool) -> (bool, DiGraph) {
    let n = graph.node_count();
    let m = graph.edge_count();

    let mut g = GraphInternal::new(n, m);
    for e in graph.edge_references() {
        let u = e.source();
        let v = e.target();
        g.add_edge(u.index(), v.index());
    }

    // root the graph, calculate low1, low2, nesting_depth, parent and height
    let mut roots = vec![];
    for u in 0..n {
        if g.height[u] == usize::MAX {
            roots.push(u);
            g.height[u] = 0;
            dfs1(&mut g, u);
        }
    }

    // sort edges inside adjacency lists according to nesting_depth
    make_adjacency_lists_acceptable(&mut g);

    // calculate LR orientation
    let mut lr_stuff = LrOrientation::new(n, m);
    for &u in &roots {
        if !dfs2(&mut g, &mut lr_stuff, u) {
            return (
                false,
                get_counterexample(graph.clone(), with_counterexample),
            );
        }
    }

    (true, embed_graph(&mut g, &mut lr_stuff, &roots))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verify_embedding(graph: &UnGraph, embedding: &DiGraph) {
        // check that edge set of the embedding matches the original graph
        for e in graph.edge_references() {
            assert!(embedding.contains_edge(
                petgraph::graph::NodeIndex::new(e.source().index()),
                petgraph::graph::NodeIndex::new(e.target().index())
            ));
            assert!(embedding.contains_edge(
                petgraph::graph::NodeIndex::new(e.target().index()),
                petgraph::graph::NodeIndex::new(e.source().index())
            ));
        }
        for e in embedding.edge_references() {
            assert!(graph.contains_edge(
                petgraph::graph::NodeIndex::new(e.source().index()),
                petgraph::graph::NodeIndex::new(e.target().index())
            ));
        }

        // TODO
    }

    fn run_test(graph: &UnGraph) {
        let (is_planar, counterexample) = is_planar(&graph, true);

        if is_planar {
            verify_embedding(&graph, &counterexample);
        }
    }

    #[cfg(all(test, not(debug_assertions)))]
    #[test]
    fn test_embedding_exhaustive() {
        use crate::testing::graph_enumerator::GraphEnumeratorState;
        use crate::testing::random_graphs::random_graph;

        for n in 2..=7 {
            let mut enumerator = GraphEnumeratorState {
                n,
                mask: 0,
                last_mask: (1 << (n * (n - 1) / 2)),
            };

            while let Some(in_graph) = enumerator.next() {
                run_test(&in_graph);
            }
        }
        for i in 0..1000 {
            let n = 2 + i / 10;
            let m: usize = 1 + i;

            let in_graph = random_graph(n, m, i);
            run_test(&in_graph);
        }
        for i in 0..1000 {
            let n = 500;
            let m = 500 + i;

            let in_graph = random_graph(n, m, i);
            run_test(&in_graph);
        }
    }
}
