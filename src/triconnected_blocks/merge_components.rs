use crate::triconnected_blocks::outside_structures::{Component, ComponentType};

/// Merges split components as much as possible.
///
/// Two components can be merged if and only if they are of the same type (excluding `R` nodes)
/// and share a common virtual edge.
pub fn merge_components(m: usize, split_components: &mut Vec<Component>) {
    let mut edge_to_component = vec![0; m];

    for (i, component) in split_components.iter().enumerate() {
        for &eid in &component.edges {
            edge_to_component[eid] = i;
        }
    }

    let mut merged_already = vec![false; split_components.len()];
    let mut ret = Vec::new();

    for (i, component) in split_components.iter().enumerate() {
        if merged_already[i] {
            continue;
        }
        if component.comp_type == ComponentType::R {
            ret.push(component.clone());
            continue;
        }

        let mut collected_edges = component.edges.clone();

        merged_already[i] = true;

        let mut j = 0;
        while j < collected_edges.len() {
            let eid = collected_edges[j];
            let other_idx = edge_to_component[eid];

            if other_idx != i
                && !merged_already[other_idx]
                && split_components[other_idx].comp_type == component.comp_type
            {
                merged_already[other_idx] = true;

                // Add all edges except the current one to avoid duplicates
                collected_edges.extend(
                    split_components[other_idx]
                        .edges
                        .iter()
                        .filter(|&&e| e != eid),
                );

                // remove the current edge, since it's invalid now
                collected_edges.swap_remove(j);
                continue;
            }
            j += 1;
        }

        if !collected_edges.is_empty() {
            let mut new_component = component.clone();
            new_component.edges = collected_edges;
            ret.push(new_component);
        }
    }

    split_components.clear();
    split_components.extend(ret);
}
