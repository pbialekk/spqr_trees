use spqr_trees::UnGraph;
use spqr_trees::drawing_blocks::triangulate::triangulate;
use spqr_trees::drawing_blocks::visualize::visualize_triangulation;
use spqr_trees::embedding::is_planar;
/// Example of triangulating a planar graph.
/// I use it with `cargo run --example triangulate | dot -Tsvg > triangulate.svg`
use spqr_trees::input::from_str;
use spqr_trees::types::DiGraph;

fn main() {
    // 0 -- 1
    // |    |
    // 3 -- 2
    let input = "
            0,1
            1,2
            2,3
            3,0
            ";
    let g_undir: UnGraph = from_str(input);
    let (is_planar, embedding) = is_planar(&g_undir, false);

    let triangulated_graph = triangulate(&g_undir);

    print!(
        "{}",
        visualize_triangulation(&embedding, &triangulated_graph)
    );
}
