/// Example of drawing spqr_tree of a graph input file.
/// I use it with `cargo run --example spqr_tree | dot -Tsvg > spqr_tree.svg`
use spqr_trees::input::from_file;
use spqr_trees::spqr_blocks::visualize::visualize_spqr;
use spqr_trees::spqr_tree::get_spqr_tree;

fn main() {
    let graph = from_file("assets/tricon.in");

    let spqr_tree = get_spqr_tree(&graph);

    print!("{}", visualize_spqr(&spqr_tree));
}
