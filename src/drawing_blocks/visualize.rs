use crate::types::DiGraph;
use petgraph::visit::EdgeRef;
use petgraph::visit::NodeIndexable;
use std::fmt::Write;

/// Generates a Graphviz DOT representation of the original and triangulated graphs side-by-side.
pub fn visualize_triangulation(original: &DiGraph, triangulated: &DiGraph) -> String {
    let mut output = String::new();

    writeln!(output, "graph triangulation {{").unwrap();
    writeln!(output, "  overlap=false;").unwrap();
    writeln!(output, "  splines=true;").unwrap();
    writeln!(output, "  bgcolor=\"#ffffff\";").unwrap();
    writeln!(output, "  node [fontname=\"Helvetica\", style=filled, fillcolor=\"#ffffff\", color=\"#333333\", penwidth=1.5];").unwrap();
    writeln!(
        output,
        "  edge [fontname=\"Helvetica\", color=\"#333333\", penwidth=1.5];"
    )
    .unwrap();
    writeln!(output).unwrap();

    write_graph_subgraph(
        &mut output,
        original,
        "cluster_original",
        "Original Graph",
        "#f9f9f9",
        "L",
        None,
    );

    writeln!(output).unwrap();

    write_graph_subgraph(
        &mut output,
        triangulated,
        "cluster_triangulated",
        "Triangulated Graph",
        "#f0f8ff",
        "R",
        Some(original),
    );

    writeln!(output, "}}").unwrap();
    output
}

/// Writes a DOT subgraph cluster for a directed graph.
///
/// If `highlight_against` is provided, edges not in that graph are highlighted in red.
fn write_graph_subgraph(
    output: &mut String,
    graph: &DiGraph,
    cluster_id: &str,
    label: &str,
    fillcolor: &str,
    prefix: &str,
    highlight_against: Option<&DiGraph>,
) {
    writeln!(output, "  subgraph {} {{", cluster_id).unwrap();
    writeln!(output, "    label=\"{}\";", label).unwrap();
    writeln!(output, "    fontname=\"Helvetica-Bold\";").unwrap();
    writeln!(output, "    fontsize=16;").unwrap();
    writeln!(output, "    color=\"#dddddd\";").unwrap();
    writeln!(output, "    style=filled; fillcolor=\"{}\";", fillcolor).unwrap();
    writeln!(output, "    margin=20;").unwrap();

    for i in 0..graph.node_count() {
        writeln!(
            output,
            "    {}_{} [label=\"{}\", width=0.4];",
            prefix,
            i,
            i + 1
        )
        .unwrap();
    }

    for e in graph.edge_references() {
        let u = graph.to_index(e.source());
        let v = graph.to_index(e.target());
        if u > v {
            continue;
        }

        if let Some(orig) = highlight_against {
            let is_new = !orig.contains_edge(orig.from_index(u), orig.from_index(v));
            let (color, width) = if is_new {
                ("#FF5733", "2.5")
            } else {
                ("#333333", "1.5")
            };
            writeln!(
                output,
                "    {}_{} -- {}_{} [color=\"{}\", penwidth={} ];",
                prefix, u, prefix, v, color, width
            )
            .unwrap();
        } else {
            writeln!(output, "    {}_{} -- {}_{};", prefix, u, prefix, v).unwrap();
        }
    }

    writeln!(output, "  }}").unwrap();
}

/// Generates an SVG representation of the graph drawn using Schnyder's algorithm.
pub fn visualize_schnyder(
    g: &DiGraph,
    drawing: &crate::drawing_blocks::schnyder::DrawingResult,
) -> String {
    let mut output = String::new();
    let width = 1000.0_f64;
    let height = 1000.0_f64;
    let padding = 50.0_f64;

    // Find coordinate bounds
    let (max_x, max_y) = drawing.coordinates.iter().fold((0.0_f64, 0.0_f64), |(mx, my), &(x, y)| {
        (mx.max(x as f64), my.max(y as f64))
    });

    // Compute scaling factors
    let scale_x = if max_x > 0.0 { (width - 2.0 * padding) / max_x } else { 1.0 };
    let scale_y = if max_y > 0.0 { (height - 2.0 * padding) / max_y } else { 1.0 };

    // Transform a graph coordinate to SVG coordinate
    let to_svg = |x: i64, y: i64| -> (f64, f64) {
        (padding + x as f64 * scale_x, height - (padding + y as f64 * scale_y))
    };

    writeln!(
        output,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\">",
        width, height
    )
    .unwrap();
    writeln!(
        output,
        "  <rect width=\"100%\" height=\"100%\" fill=\"white\" />"
    )
    .unwrap();

    // Draw grid
    let grid_step = if max_x > 20.0 || max_y > 20.0 { 5.0 } else { 1.0 };

    writeln!(output, "  <g stroke=\"#999\" stroke-width=\"1\">").unwrap();
    // Horizontal grid lines
    let mut y = 0.0;
    while y <= max_y {
        let (sx_start, sy) = (padding, to_svg(0, y as i64).1);
        writeln!(
            output,
            "    <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" />",
            sx_start, sy, width - padding, sy
        )
        .unwrap();
        y += grid_step;
    }
    // Vertical grid lines
    let mut x = 0.0;
    while x <= max_x {
        let (sx, _) = to_svg(x as i64, 0);
        writeln!(
            output,
            "    <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" />",
            sx, height - padding, sx, padding
        )
        .unwrap();
        x += grid_step;
    }
    writeln!(output, "  </g>").unwrap();

    // Draw edges
    for (u, v, color) in &drawing.edge_colors {
        let (x1, y1) = drawing.coordinates[*u];
        let (x2, y2) = drawing.coordinates[*v];
        let (sx1, sy1) = to_svg(x1, y1);
        let (sx2, sy2) = to_svg(x2, y2);

        let stroke_color = match color {
            crate::drawing_blocks::schnyder::Color::Red => "red",
            crate::drawing_blocks::schnyder::Color::Blue => "blue",
            crate::drawing_blocks::schnyder::Color::Green => "green",
            crate::drawing_blocks::schnyder::Color::Black => "black",
        };

        writeln!(
            output,
            "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"2\" marker-end=\"url(#arrow)\"/>",
            sx1, sy1, sx2, sy2, stroke_color
        )
        .unwrap();
    }

    // Definitions for markers
    writeln!(output, "  <defs>").unwrap();
    writeln!(output, "    <marker id=\"arrow\" markerWidth=\"10\" markerHeight=\"10\" refX=\"18\" refY=\"3\" orient=\"auto\" markerUnits=\"strokeWidth\">").unwrap();
    writeln!(
        output,
        "      <path d=\"M0,0 L0,6 L9,3 z\" fill=\"#555\" />"
    )
    .unwrap();
    writeln!(output, "    </marker>").unwrap();
    writeln!(output, "  </defs>").unwrap();

    // Draw nodes on top of edges
    for i in 0..g.node_count() {
        let (x, y) = drawing.coordinates[i];
        let (sx, sy) = to_svg(x, y);

        writeln!(
            output,
            "  <circle cx=\"{}\" cy=\"{}\" r=\"6\" fill=\"black\" />",
            sx, sy
        )
        .unwrap();
        writeln!(output, "  <text x=\"{}\" y=\"{}\" font-family=\"Arial\" font-size=\"12\" fill=\"white\" text-anchor=\"middle\" dy=\".3em\">{}</text>", sx, sy, i).unwrap();
    }

    writeln!(output, "</svg>").unwrap();
    output
}
