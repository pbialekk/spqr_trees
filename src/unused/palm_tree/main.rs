/// Example of drawing a palm tree from a graph input file.
/// I use it with `cargo run --example palm_tree | dot -Tsvg > palm.svg`
use spqr_trees::input::from_file;
use spqr_trees::palm_tree::{draw_palm_tree, get_palm_tree};

fn main() {
    let graph = from_file("assets/posch.in");

    let palm_tree = get_palm_tree(&graph);

    print!("{}", draw_palm_tree(&palm_tree, &graph))
}
