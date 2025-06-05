//! I use it with `cargo run --example palm_tree | dot -Tsvg > palm.svg`

use spqr_trees::{from_file, get_palm_tree, draw_palm_tree};

fn main() {
    let graph = from_file("assets/in1.graph");

    let palm_tree = get_palm_tree(&graph);

    // for node_idx in graph.node_indices() {
    //     let label = graph.node_weight(node_idx).unwrap();
    //     println!("Node {}: {:?}", node_idx.index(), label);
    // }

    // println!("{:?}", palm_tree)

    print!("{}", draw_palm_tree(&palm_tree, &graph))
}
