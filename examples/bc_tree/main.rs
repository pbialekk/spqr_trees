/// Example of drawing a block-cut tree from a graph input file.
/// I use it with `cargo run --example bc_tree | neato -Tsvg > bc.svg`

use spqr_trees::input::from_file;
use spqr_trees::block_cut::{get_block_cut_tree, draw_skeleton_of_block_cut_tree_dot};

fn main() {
    let graph = from_file("assets/bc.in");

    let bc_tree = get_block_cut_tree(&graph);

    print!("{}", draw_skeleton_of_block_cut_tree_dot(&bc_tree));
}