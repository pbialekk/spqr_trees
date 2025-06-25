//! I use it with `cargo run --example output | dot -Tsvg > graph.svg`

use spqr_trees::input::from_file;
use spqr_trees::output::to_dot_str;

fn main() {
    let graph = from_file("assets/graph.in");
    print!("{}", to_dot_str(&graph))
}