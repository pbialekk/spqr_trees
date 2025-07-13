/// Example of drawing triconnected components of a graph input file.
/// I use it with `cargo run --example split_components | dot -Tsvg > split_components.svg`
use spqr_trees::input::from_file;
use spqr_trees::triconnected::get_triconnected_components;
use spqr_trees::triconnected_blocks::visualize::visualize_triconnected;

fn main() {
    let graph = from_file("assets/tricon.in");

    let split_components = get_triconnected_components(&graph);

    print!("{}", visualize_triconnected(&split_components));
}
