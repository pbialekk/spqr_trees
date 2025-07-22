/// Example of drawing triconnected components of envelope graph.
/// I use it with `cargo run --example spqr_tree_envelope | dot -Tsvg > spqr_tree_envelope.svg`
use spqr_trees::EdgeLabel;
use spqr_trees::UnGraph;
use spqr_trees::spqr_blocks::visualize::visualize_spqr;
use spqr_trees::spqr_tree::get_spqr_tree;

fn main() {
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

    let spqr = get_spqr_tree(&graph);

    print!("{}", visualize_spqr(&spqr));
}
