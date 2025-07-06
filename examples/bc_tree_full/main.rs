/// Example of drawing a full block-cut tree from a graph input file.
/// I use it with `cargo run --example bc_tree_full | neato -Tsvg > bc_full.svg`

use spqr_trees::input::from_file;
use spqr_trees::block_cut::{get_block_cut_tree, draw_full_block_cut_tree};

fn main() {
    let graph = from_file("assets/bc.in");

    let bc_tree = get_block_cut_tree(&graph);

    print!("{}", draw_full_block_cut_tree(&bc_tree));
}