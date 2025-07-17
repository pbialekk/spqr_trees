/// Example of drawing triconnected components of a graph input file.
/// I use it with `cargo run --example spqr_tree | dot -Tsvg > spqr_tree.svg`
use spqr_trees::input::from_file;
use spqr_trees::spqr_blocks::visualize::visualize_spqr;
use spqr_trees::spqr_tree::get_spqr_tree;
use spqr_trees::testing::random_graphs::random_biconnected_graph;

fn main() {
    let n = 10000;
    let m: usize = 10 * n;

    let in_graph = random_biconnected_graph(n, m, 12345);

    let split_components = get_spqr_tree(&in_graph);

    print!("{}", visualize_spqr(&split_components));
}
