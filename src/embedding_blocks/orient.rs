use std::mem;

use crate::embedding_blocks::structures::GraphInternal;

/// Implements the DFS1 algorithm from the reference.
pub fn dfs1(graph: &mut GraphInternal, u: usize) {
    let neighbors = graph.adj[u].clone();

    for &eid in neighbors.iter() {
        if graph.low1[eid] != usize::MAX {
            // a back-edge towards us, skip it
            continue;
        }

        let to = graph.get_other_vertex(eid, u);
        if graph.edges[eid].0 == to {
            // swap
            let edge = &mut graph.edges[eid];
            mem::swap(&mut edge.0, &mut edge.1);
        }

        graph.low1[eid] = graph.height[u];
        graph.low2[eid] = graph.height[u];

        if graph.height[to] == usize::MAX {
            graph.parent[to] = Some(eid);
            graph.height[to] = graph.height[u] + 1;
            dfs1(graph, to);
        } else {
            graph.low1[eid] = graph.height[to];
        }

        graph.nesting_depth[eid] = 2 * graph.low1[eid] as isize;
        if graph.low2[eid] < graph.height[u] {
            // chordal edge
            graph.nesting_depth[eid] += 1;
        }

        if let Some(eid_par) = graph.parent[u] {
            if graph.low1[eid] < graph.low1[eid_par] {
                graph.low2[eid_par] = graph.low1[eid_par].min(graph.low2[eid]);
                graph.low1[eid_par] = graph.low1[eid];
            } else if graph.low1[eid] != graph.low1[eid_par] {
                graph.low2[eid_par] = graph.low2[eid_par].min(graph.low1[eid]);
            } else {
                graph.low2[eid_par] = graph.low2[eid_par].min(graph.low2[eid]);
            }
        }
    }
}
