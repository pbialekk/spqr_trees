/// Example of outputting a graph in DOT format.
/// I use it with `cargo run --example output | neato -Tsvg > posch.svg`
use spqr_trees::input::from_file;
use spqr_trees::output::draw_graph;

fn main() {
    let graph = from_file("assets/posch.in");
    print!("{}", draw_graph(&graph))
}