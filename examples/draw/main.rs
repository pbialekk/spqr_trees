use spqr_trees::UnGraph;
use spqr_trees::drawing_blocks::schnyder::draw;
use spqr_trees::drawing_blocks::triangulate::triangulate;
use spqr_trees::drawing_blocks::visualize::visualize_schnyder;
/// Example of drawing a graph using Schnyder's algorithm.
/// Usage: `cargo run --example draw > drawing.svg`
use spqr_trees::input::from_str;

fn main() {
    // 0 -- 1
    // |    |
    // 3 -- 2
    let input = "
            0,1
            1,2
            2,3
            3,4
            4,5
            5,0
            ";

    let g_undir: UnGraph = from_str(input);

    // Triangulate
    let triangulated_graph = triangulate(&g_undir);

    // Draw
    let drawing = draw(&triangulated_graph);

    // Visualize
    print!("{}", visualize_schnyder(&triangulated_graph, &drawing));
}
