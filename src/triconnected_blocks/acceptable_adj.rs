use crate::triconnected_blocks::{graph_internal::GraphInternal, outside_structures::EdgeType};

/// Given a graph g, this function modifies it's adjacency lists.
/// Each edge is assigned a value phi(e) and the edges are then
/// sorted inside the adjacency lists based on these values.
///
/// The phi function is defined as follows:
/// - If the edge (`e = (u, to)`) is a tree edge and the lowpoint of the target vertex is less than the discovery time of the source vertex,
///   then `phi(e) = 3 * low1[to]` (there's only one way to escape the subtree rooted at `to`)
/// - If the edge is a tree edge and the lowpoint of the target vertex is greater than or equal to the discovery time of the source vertex,
///   then `phi(e) = 3 * low1[to] + 2` (there is more than one way to escape the subtree rooted at `to`)
/// - If the edge is not a tree edge, then `phi(e) = 3 * num[to] + 1`
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
