use spqr_trees::UnGraph;
use spqr_trees::drawing_blocks::triangulate::triangulate;
use spqr_trees::drawing_blocks::visualize::visualize_triangulation;
use spqr_trees::embedding::is_planar;
/// Usage: `cargo run --example triangulate | dot -Tsvg > drawing.svg`
use spqr_trees::input::from_str;

fn main() {
    // 0 -- 1
    // |    |
    // 3 -- 2
    let input = "
            0,2
            0,4
            0,5
            1,4
            1,5
            2,3
            2,4
            4,5
            ";
    // Or read from stdin if you prefer:
    // let mut buffer = String::new();
    // std::io::stdin().read_to_string(&mut buffer).unwrap();
    // let g_undir: UnGraph = from_str(&buffer);

    let g_undir: UnGraph = from_str(input);

    // Triangulate
    let triangulated_graph = triangulate(&g_undir);

    let (_, original_embedded) = is_planar(&g_undir, false);

    print!(
        "{}",
        visualize_triangulation(&original_embedded, &triangulated_graph)
    );
}
