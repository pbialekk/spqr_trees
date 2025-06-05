//! I use it with `cargo run --example palm_tree | dot -Tsvg > palm.svg`

use spqr_trees::{from_file, get_palm_tree, draw_palm_tree};

fn main() {
    let graph = from_file("assets/graph.in");

    let palm_tree = get_palm_tree(&graph);

    print!("{}", draw_palm_tree(&palm_tree, &graph))
}
