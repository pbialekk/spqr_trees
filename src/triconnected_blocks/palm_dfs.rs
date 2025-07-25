use std::mem;

use crate::triconnected_blocks::{graph_internal::GraphInternal, outside_structures::EdgeType};

fn dfs(u: usize, time: &mut usize, graph: &mut GraphInternal) {
    graph.num[u] = *time;
    graph.low1[u] = *time;
    graph.low2[u] = *time;
    graph.sub[u] = 1;
    *time += 1;

    let neighbors = graph.adj[u].clone(); // borrow checker doesn't like mutable borrow below

    for &eid in &neighbors {
        let to = graph.get_other_vertex(eid, u);

        if graph.edge_type[eid].is_some() {
            continue; // already visited 
        }

        if graph.num[to] == usize::MAX {
            // tree edge
            graph.par_edge[to] = Some(eid);
            graph.par[to] = Some(u);
            graph.edge_type[eid] = Some(EdgeType::Tree);

            dfs(to, time, graph);

            graph.sub[u] += graph.sub[to];

            if graph.low1[to] < graph.low1[u] {
                graph.low2[u] = graph.low1[u].min(graph.low2[to]);
                graph.low1[u] = graph.low1[to];
            } else if graph.low1[to] == graph.low1[u] {
                graph.low2[u] = graph.low2[u].min(graph.low2[to]);
            } else {
                graph.low2[u] = graph.low2[u].min(graph.low1[to]);
            }
        } else {
            // back edge (upwards)
            graph.edge_type[eid] = Some(EdgeType::Back);

            if graph.num[to] < graph.low1[u] {
                graph.low2[u] = graph.low1[u];
                graph.low1[u] = graph.num[to];
            } else if graph.num[to] > graph.low1[u] {
                graph.low2[u] = graph.low2[u].min(graph.num[to]);
            }
        }
    }
}

/// Computes the palm tree decomposition of the given graph using a depth-first search (DFS).
///
/// This function calculates and populates the following for each vertex:
/// - `num[u]`: The preorder number (DFS visitation order) of vertex `u`.
/// - `low1[u]`: The lowest `num` reachable from `u` via tree edges (0 or more) followed by exactly 1 `Back` edge.
/// - `low2[u]`: The second lowest `num` reachable from `u` in the same manner as `low1[u]`.
/// - `sub[u]`: The size of the subtree rooted at `u`.
///
/// Additionally, it classifies each edge as either a `Tree` edge or a `Back` edge,
/// updating the `edge_type` field in the graph accordingly. After the DFS,
/// all edges in `graph.edges` are oriented from source to target.
///
/// The idea is pretty simple: we run DFS and we update `low1` and `low2` when we can
pub fn run_palm_dfs(graph: &mut GraphInternal, root: usize) {
    let mut time = 0;
    dfs(root, &mut time, graph);

    // now that for each edge we know its type, we can assure that edges in `edges` always point from source to target
    for (eid, edge) in graph.edges.iter_mut().enumerate() {
        let (s, t) = (edge.0, edge.1);
        if (graph.edge_type[eid] == Some(EdgeType::Back) && graph.num[s] < graph.num[t])
            || (graph.edge_type[eid] == Some(EdgeType::Tree) && graph.num[s] > graph.num[t])
        {
            mem::swap(&mut edge.0, &mut edge.1);
        }
    }
}
