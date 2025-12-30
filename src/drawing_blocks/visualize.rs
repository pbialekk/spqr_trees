use crate::UnGraph;
use crate::drawing_blocks::faces::get_faces;
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
