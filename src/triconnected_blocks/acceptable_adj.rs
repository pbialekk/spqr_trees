use crate::triconnected_blocks::{graph_internal::GraphInternal, outside_structures::EdgeType};

/// Modifies the adjacency lists of the given graph so that edges are sorted
/// according to a custom phi value.
///
/// The phi function assigns a value to each edge as follows:
/// - For a tree edge (e = (u, to)):
///     - If `low2[to] < num[u]`, then `phi(e) = 3 * low1[to]`
///       (only one way to escape the subtree rooted at `to`)
///     - Otherwise, `phi(e) = 3 * low1[to] + 2`
///       (more than one way to escape the subtree rooted at `to`)
/// - For a non-tree edge: `phi(e) = 3 * num[to] + 1`
///
/// Edges are then bucket-sorted by their phi values and adjacency lists are rebuilt.
pub(crate) fn make_adjacency_lists_acceptable(graph: &mut GraphInternal) {
    let phi = |eid: usize| -> usize {
        let (u, to) = graph.edges[eid];
        if graph.edge_type[eid] == Some(EdgeType::Tree) {
            if graph.low2[to] < graph.num[u] {
                3 * graph.low1[to]
            } else {
                3 * graph.low1[to] + 2
            }
        } else {
            3 * graph.num[to] + 1
        }
    };

    let n = graph.n;

    // A simple bucket sort implementation
    let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); 3 * (n - 1) + 2 + 1];

    for (eid, edge) in graph.edges.iter().enumerate() {
        if graph.edge_type[eid] == Some(EdgeType::Killed) {
            continue; // skip killed edges
        }

        let edge_value = phi(eid);
        buckets[edge_value].push(eid);
    }

    let mut new_adj = vec![vec![]; n];
    for bucket in buckets {
        for eid in bucket {
            let (s, t) = graph.edges[eid];
            new_adj[s].push(eid);
        }
    }

    graph.adj = new_adj;
}
