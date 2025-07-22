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

    fn verify_embedding(embedding: &DiGraph, output: &mut String) {
        let mut edges = String::new();

        for v in embedding.node_indices() {
            edges.push_str(&format!("{}", v.index()));
            for n in embedding.neighbors(v) {
                edges.push_str(&format!(" {}", n.index()));
            }
            edges.push_str("\n");
        }
        output.push_str("+\n");
        output.push_str(&edges);
    }

    fn run_test(graph: &UnGraph, output: &mut String) {
        let mut edges = String::new();
        for e in graph.edge_references() {
            edges.push_str(&format!("{},{}\n", e.source().index(), e.target().index()));
        }
        output.push_str(&edges);

        let (is_planar, counterexample) = is_planar(&graph, true);

        if is_planar {
            verify_embedding(&counterexample, output);
        } else {
            output.push_str("-\n");
        }
    }

    #[cfg(all(test, not(debug_assertions)))]
    #[test]
    fn test_embedding_exhaustive() {
        // Generate test, write it to a file, run python script and verify with our answer
        use crate::testing::graph_enumerator::GraphEnumeratorState;
        use crate::testing::random_graphs::random_graph;

        let mut python_input = String::new();

        for n in 2..=7 {
            let mut enumerator = GraphEnumeratorState {
                n,
                mask: 0,
                last_mask: (1 << (n * (n - 1) / 2)),
            };

            while let Some(in_graph) = enumerator.next() {
                run_test(&in_graph, &mut python_input);
            }
        }
        for i in 0..1000 {
            let n = 2 + i / 10;
            let m: usize = 1 + i;

            let in_graph = random_graph(n, m, i);
            run_test(&in_graph, &mut python_input);
        }
        for i in 0..1000 {
            let n = 500;
            let m = 500 + i;

            let in_graph = random_graph(n, m, i);
            run_test(&in_graph, &mut python_input);
        }

        std::fs::write("assets/python_input.in", python_input).expect("Unable to write file");
        let output = std::process::Command::new("python3")
            .arg("assets/verify_answers.py")
            .output()
            .expect("Failed to execute python script");

        if !output.status.success() {
            panic!(
                "Python script failed with error: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
