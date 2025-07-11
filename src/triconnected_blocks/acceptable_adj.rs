use crate::triconnected_blocks::{graph_internal::GraphInternal, outside_structures::EdgeType};

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
