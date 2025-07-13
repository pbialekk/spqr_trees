use crate::{
    spqr_blocks::outside_structures::SPQRTree,
    triconnected_blocks::visualize::visualize_triconnected,
};

pub fn visualize_spqr(spqr: &SPQRTree) -> String {
    let mut output = visualize_triconnected(&spqr.triconnected_components);

    // if a line ends in "  }", it's an end of a subgraph. iterate over them

    let mut i = 0;

    let mut j = 0;
    while let Some(new_j) = output[j..].find("  }") {
        if j == 0 {
            // this is a real graph, skip it.
            j += new_j + 3; // skip "  }"
            continue;
        }
        let prefix = spqr.triconnected_components.components[i]
            .component_type
            .clone()
            .unwrap();
        let prefix = prefix.to_string();
        let label = format!("{}{}_connector", prefix, i + 1);

        let write_str = format!(
            "    {} [shape=point, width=0.1, label=\"\", color=black];\n  }}",
            label
        );

        let abs_j = j + new_j;
        output.replace_range(abs_j..abs_j + 3, &write_str);
        j = abs_j + write_str.len();
        i += 1;
    }

    // remove the last "  }"
    output.truncate(output.len() - 4);

    // add a newline
    output.push('\n');

    // and add spqr edges

    for (u, adj_u) in spqr.adj.iter().enumerate() {
        for &v in adj_u {
            if u < v {
                let u_type = spqr.triconnected_components.components[u].component_type;
                let v_type = spqr.triconnected_components.components[v].component_type;

                let u_prefix = u_type.clone().unwrap().to_string();
                let v_prefix = v_type.clone().unwrap().to_string();

                let u_label = format!("{}{}_connector", u_prefix, u + 1);
                let v_label = format!("{}{}_connector", v_prefix, v + 1);

                let u_cluster = format!("cluster_{}{}", u_prefix, u + 1);
                let v_cluster = format!("cluster_{}{}", v_prefix, v + 1);

                let edge_str = format!(
                    "  {} -- {} [ltail={}, lhead={}, color=black, penwidth=0.2];\n",
                    u_label, v_label, u_cluster, v_cluster
                );

                output.push_str(&edge_str);
            }
        }
    }

    output.push_str("}\n");

    output
}
