/// Example of drawing a DFS tree when computing a block-cut tree.
/// I use it with `cargo run --example bc_dfs | dot -Tsvg > bc_dfs.svg`

use spqr_trees::input::from_file;
use spqr_trees::block_cut::{get_block_cut_tree, draw_bc_tree_dfs};

fn main() {
    let graph = from_file("assets/bc.in");

    let bc_tree = get_block_cut_tree(&graph);

    print!("{}", draw_bc_tree_dfs(&graph, &bc_tree));
}