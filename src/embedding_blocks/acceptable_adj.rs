use crate::embedding_blocks::structures::GraphInternal;

/// Sorts the adjacency lists of the graph according to the nesting depth of edges.
pub fn make_adjacency_lists_acceptable(graph: &mut GraphInternal) {
    let max_phi = 2 * (graph.n - 1) + 1;

    let phi = |eid: usize| -> usize { (max_phi as isize + graph.nesting_depth[eid]) as usize };

    // A simple bucket sort implementation
    let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); 2 * max_phi + 1];

    for eid in 0..graph.m {
        let edge_value = phi(eid);
        buckets[edge_value as usize].push(eid);
    }

    let mut new_adj = vec![vec![]; graph.n];
    for bucket in buckets {
        for eid in bucket {
            let (s, _) = graph.edges[eid];
            new_adj[s].push(eid);
        }
    }

    graph.adj = new_adj;
}
