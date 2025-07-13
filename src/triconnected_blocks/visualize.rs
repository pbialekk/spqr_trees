use crate::triconnected_blocks::outside_structures::{ComponentType, TriconnectedComponents};
use std::fmt::Write;

pub fn visualize_triconnected(tricon: &TriconnectedComponents) -> String {
    let mut output = String::new();

    writeln!(output, "graph components {{").unwrap();
    writeln!(output, "  graph [splines=true, rankdir=LR];").unwrap();
    writeln!(output, "  node [fontname=\"Helvetica\"];").unwrap();
    writeln!(output).unwrap();

    for (i, comp) in tricon.components.iter().enumerate() {
        let (prefix, label, fillcolor, nodecolor) = match comp.component_type {
            Some(ComponentType::R) => (
                "R",
                format!("R-component ({})", i + 1),
                "#e6e6ff",
                "#ccccff",
            ),
            Some(ComponentType::P) => (
                "P",
                format!("P-component ({})", i + 1),
                "#e6ffe6",
                "#ccffcc",
            ),
            Some(ComponentType::S) => (
                "S",
                format!("S-component ({})", i + 1),
                "#ffe6e6",
                "#ffcccc",
            ),
            _ => {
                panic!();
            }
        };

        writeln!(output, "  subgraph cluster_{}{} {{", prefix, i + 1).unwrap();
        writeln!(output, "    label=\"{}\";", label).unwrap();
        writeln!(output, "    style=filled; fillcolor=\"{}\";", fillcolor).unwrap();

        let mut nodes = Vec::new();
        for &v in &comp.edges {
            let (from, to) = tricon.edges[v];
            if !nodes.contains(&from) {
                nodes.push(from);
            }
            if !nodes.contains(&to) {
                nodes.push(to);
            }
        }

        // Nodes
        for v in nodes {
            writeln!(
                output,
                "    {}{}_{} [label=\"{}\", shape=circle, fillcolor=\"{}\", style=filled];",
                prefix,
                i + 1,
                v,
                v,
                nodecolor
            )
            .unwrap();
        }
        writeln!(output).unwrap();

        // Edges
        for e in &comp.edges {
            let (from, to, label, is_virtual) = (
                tricon.edges[*e].0,
                tricon.edges[*e].1,
                *e,
                !tricon.is_real_edge[*e],
            );
            writeln!(
                output,
                "    {}{}_{} -- {}{}_{} [label=\"{}\"{}];",
                prefix,
                i + 1,
                from,
                prefix,
                i + 1,
                to,
                label,
                if is_virtual {
                    ", style=dashed, color=gray"
                } else {
                    ", color=black"
                }
            )
            .unwrap();
        }

        writeln!(output, "  }}").unwrap();
        writeln!(output).unwrap();
    }

    writeln!(output, "}}").unwrap();
    output
}
