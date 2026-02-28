use crate::triconnected_blocks::outside_structures::TriconnectedComponents;
use hashbrown::HashSet;
use std::fmt::Write;

/// Collects unique vertex indices from a set of edge indices.
fn collect_nodes(edges: &[usize], all_edges: &[(usize, usize)]) -> Vec<usize> {
    let mut seen = HashSet::new();
    let mut nodes = Vec::new();
    for &eid in edges {
        let (from, to) = all_edges[eid];
        for v in [from, to] {
            if seen.insert(v) {
                nodes.push(v);
            }
        }
    }
    nodes
}

/// Given a `TriconnectedComponents` structure, this function generates a
/// Graphviz DOT representation of the triconnected components of a graph.
pub fn visualize_triconnected(tricon: &TriconnectedComponents) -> String {
    let mut output = String::new();

    writeln!(output, "graph components {{").unwrap();
    writeln!(output, "  graph [splines=true, rankdir=LR, compound=true];").unwrap();
    writeln!(output, "  node [fontname=\"Helvetica\"];").unwrap();
    writeln!(output).unwrap();

    {
        writeln!(output, "  // The actual graph").unwrap();
        writeln!(output, "  subgraph cluster_graph {{").unwrap();
        writeln!(output, "    label=\"Graph\";").unwrap();
        writeln!(output, "    style=filled; fillcolor=\"#f0f0f0\";").unwrap();

        let real_eids: Vec<usize> = (0..tricon.edges.len()).filter(|&i| tricon.is_real[i]).collect();
        let nodes = collect_nodes(&real_eids, &tricon.edges);

        for v in &nodes {
            writeln!(
                output,
                "    {} [label=\"{}\", shape=circle, fillcolor=\"#ffffff\", style=filled];",
                v, v
            )
            .unwrap();
        }
        writeln!(output).unwrap();

        for &eid in &real_eids {
            let (from, to) = tricon.edges[eid];
            writeln!(
                output,
                "    {} -- {} [label=\"{}\", color=black];",
                from, to, eid
            )
            .unwrap();
        }

        writeln!(output, "  }}").unwrap();
        writeln!(output).unwrap();
    }

    for (i, comp) in tricon.comp.iter().enumerate() {
        let (fillcolor, nodecolor, prefix) = comp.comp_type.vis_colors();
        let label = format!("{}-component ({})", prefix, i + 1);

        writeln!(output, "  subgraph cluster_{}{} {{", prefix, i + 1).unwrap();
        writeln!(output, "    label=\"{}\";", label).unwrap();
        writeln!(output, "    style=filled; fillcolor=\"{}\";", fillcolor).unwrap();

        let nodes = collect_nodes(&comp.edges, &tricon.edges);

        for v in &nodes {
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

        for e in &comp.edges {
            let (from, to) = tricon.edges[*e];
            let style = if tricon.is_real[*e] {
                ", color=black"
            } else {
                ", style=dashed, color=gray"
            };
            writeln!(
                output,
                "    {}{}_{} -- {}{}_{} [label=\"{}\"{}];",
                prefix, i + 1, from, prefix, i + 1, to, e, style
            )
            .unwrap();
        }

        writeln!(output, "  }}").unwrap();
        writeln!(output).unwrap();
    }

    writeln!(output, "}}").unwrap();
    output
}
