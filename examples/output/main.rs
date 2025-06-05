//! I use it with `cargo run --example output | dot -Tsvg > in1.svg`

use spqr_trees::from_file;
use spqr_trees::to_dot_str;

fn main() {
    let graph = from_file("assets/in1.graph");
    print!("{}", to_dot_str(&graph))
}