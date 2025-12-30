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

    {
        writeln!(output, "  subgraph cluster_original {{").unwrap();
        writeln!(output, "    label=\"Original Graph\";").unwrap();
        writeln!(output, "    fontname=\"Helvetica-Bold\";").unwrap();
        writeln!(output, "    fontsize=16;").unwrap();
        writeln!(output, "    color=\"#dddddd\";").unwrap();
        writeln!(output, "    style=filled; fillcolor=\"#f9f9f9\";").unwrap();
        writeln!(output, "    margin=20;").unwrap();

        let prefix = "L";

        // Nodes
        for i in 0..original.node_count() {
            writeln!(
                output,
                "    {}_{} [label=\"{}\", width=0.4];",
                prefix,
                i,
                i + 1
            )
            .unwrap();
        }

        // Edges
        for e in original.edge_references() {
            let u = original.to_index(e.source());
            let v = original.to_index(e.target());
            if u > v {
                continue;
            }
            writeln!(output, "    {}_{} -- {}_{};", prefix, u, prefix, v).unwrap();
        }

        writeln!(output, "  }}").unwrap();
    }

    writeln!(output).unwrap();

    {
        writeln!(output, "  subgraph cluster_triangulated {{").unwrap();
        writeln!(output, "    label=\"Triangulated Graph\";").unwrap();
        writeln!(output, "    fontname=\"Helvetica-Bold\";").unwrap();
        writeln!(output, "    fontsize=16;").unwrap();
        writeln!(output, "    color=\"#dddddd\";").unwrap();
        writeln!(output, "    style=filled; fillcolor=\"#f0f8ff\";").unwrap(); // AliceBlue
        writeln!(output, "    margin=20;").unwrap();

        let prefix = "R";

        // Nodes
        for i in 0..triangulated.node_count() {
            writeln!(
                output,
                "    {}_{} [label=\"{}\", width=0.4];",
                prefix,
                i,
                i + 1
            )
            .unwrap();
        }

        // Edges
        for e in triangulated.edge_references() {
            let u = triangulated.to_index(e.source());
            let v = triangulated.to_index(e.target());
            if u > v {
                continue;
            }

            let is_new = !original.contains_edge(original.from_index(u), original.from_index(v));

            let (color, width, style) = if is_new {
                ("#FF5733", "2.5", "")
            } else {
                ("#333333", "1.5", "")
            };

            writeln!(
                output,
                "    {}_{} -- {}_{} [color=\"{}\", penwidth={} {}];",
                prefix, u, prefix, v, color, width, style
            )
            .unwrap();
        }

        writeln!(output, "  }}").unwrap();
    }
    writeln!(output, "}}").unwrap();
    output
}

/// Generates an SVG representation of the graph drawn using Schnyder's algorithm.
pub fn visualize_schnyder(
    g: &DiGraph,
    drawing: &crate::drawing_blocks::schnyder::DrawingResult,
) -> String {
    let mut output = String::new();
    let width = 1000.0;
    let height = 1000.0;
    let padding = 50.0;

    // Find bounds
    let mut max_x = 0.0;
    let mut max_y = 0.0;
    for &(x, y) in &drawing.coordinates {
        let x = x as f64;
        let y = y as f64;
        if x > max_x {
            max_x = x;
        }
        if y > max_y {
            max_y = y;
        }
    }

    // Scale
    let scale_x = if max_x > 0.0 {
        (width - 2.0 * padding) / max_x
    } else {
        1.0
    };
    let scale_y = if max_y > 0.0 {
        (height - 2.0 * padding) / max_y
    } else {
        1.0
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
    let _grid_step_x = max_x.max(1.0) / 10.0;
    let _grid_step_y = max_y.max(1.0) / 10.0; // Draw 10 lines roughly

    // Draw simple grid lines (integers)
    writeln!(output, "  <g stroke=\"#999\" stroke-width=\"1\">").unwrap();
    // Horizontal
    let mut y = 0.0;
    while y <= max_y {
        let sy = height - (padding + y * scale_y);
        let sx_start = padding;
        let sx_end = width - padding;
        writeln!(
            output,
            "    <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" />",
            sx_start, sy, sx_end, sy
        )
        .unwrap();
        y += 1.0;
        if max_y > 20.0 {
            y += 4.0;
        } // Skip if too dense
    }
    // Vertical
    let mut x = 0.0;
    while x <= max_x {
        let sx = padding + x * scale_x;
        let sy_start = height - padding;
        let sy_end = padding;
        writeln!(
            output,
            "    <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" />",
            sx, sy_start, sx, sy_end
        )
        .unwrap();
        x += 1.0;
        if max_x > 20.0 {
            x += 4.0;
        }
    }
    writeln!(output, "  </g>").unwrap();

    // Draw edges
    for (u, v, color) in &drawing.edge_colors {
        let (x1, y1) = drawing.coordinates[*u];
        let (x2, y2) = drawing.coordinates[*v];

        let x1 = x1 as f64;
        let y1 = y1 as f64;
        let x2 = x2 as f64;
        let y2 = y2 as f64;

        let sx1 = padding + x1 * scale_x;
        let sy1 = height - (padding + y1 * scale_y); // Flip Y for SVG
        let sx2 = padding + x2 * scale_x;
        let sy2 = height - (padding + y2 * scale_y);

        let stroke_color = match color {
            crate::drawing_blocks::schnyder::Color::Red => "red",
            crate::drawing_blocks::schnyder::Color::Blue => "blue",
            crate::drawing_blocks::schnyder::Color::Green => "green",
            crate::drawing_blocks::schnyder::Color::Black => "black",
        };

        // Draw arrow?
        // Simple line for now, maybe finding midpoint for arrow?
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

    // Redraw edges with markers if needed? I didn't add the attribute above.
    // Let's just draw nodes on top.

    // Draw nodes
    for i in 0..g.node_count() {
        let (x, y) = drawing.coordinates[i];
        let x = x as f64;
        let y = y as f64;
        let sx = padding + x * scale_x;
        let sy = height - (padding + y * scale_y);

        writeln!(
            output,
            "  <circle cx=\"{}\" cy=\"{}\" r=\"6\" fill=\"black\" />",
            sx, sy
        )
        .unwrap();
        // ID label
        writeln!(output, "  <text x=\"{}\" y=\"{}\" font-family=\"Arial\" font-size=\"12\" fill=\"white\" text-anchor=\"middle\" dy=\".3em\">{}</text>", sx, sy, i).unwrap();
    }

    writeln!(output, "</svg>").unwrap();
    output
}
