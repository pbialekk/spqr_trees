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

/// Given a graph, this function calculates the `palm tree` of the graph using a DFS algorithm.
///
/// In particular, it calculates the values needed further in the algorithm:
/// - `num[u]` - the order of the vertex in the DFS traversal (preorder number)
/// - `low1[u]` - the lowest `num` value reachable from `u` via tree edges
/// - `low2[u]` - the second lowest `num` value reachable
/// - `sub[u]` - the size of the subtree rooted at `u`
///
/// It also determines the type of each edge in the graph, which can be either `Tree` or `Back`.
///
/// The function modifies the `graph` in place, setting the `edge_type` for each edge and ensuring that edges always point from source to target.
pub(crate) fn run_palm_dfs(graph: &mut GraphInternal, root: usize) {
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
