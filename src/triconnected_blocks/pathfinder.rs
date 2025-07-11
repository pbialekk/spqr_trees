use crate::triconnected_blocks::{graph_internal::GraphInternal, outside_structures::EdgeType};

fn dfs(
    root: usize,
    u: usize,
    graph: &mut GraphInternal,
    newnum: &mut Vec<usize>,
    time: &mut usize,
) {
    let first_to = graph.first_alive(root, u);

    let neighbors = graph.adj[u].clone(); // borrow checker doesn't like mutable borrow below

    for &eid in neighbors.iter() {
        let to = graph.get_other_vertex(eid, u);

        if Some(to) != first_to {
            graph.starts_path[eid] = true;
        }

        if graph.edge_type[eid] == Some(EdgeType::Tree) {
            dfs(root, to, graph, newnum, time);
        } else {
            // always a back edge
            graph.high[to].push(eid);
        }
    }

    newnum[u] = *time;
    *time = time.saturating_sub(1);
}

pub(crate) fn run_pathfinder(root: usize, graph: &mut GraphInternal) {
    let mut newnum = vec![0; graph.n];
    let mut time = graph.n - 1;
    dfs(root, root, graph, &mut newnum, &mut time);

    // now we need to renumber the vertices from num(v) to newnum(v)
    let mut num2newnum = vec![0; graph.n];
    for u in 0..graph.n {
        num2newnum[graph.num[u]] = newnum[u];
    }

    for u in 0..graph.n {
        graph.low1[u] = num2newnum[graph.low1[u]];
        graph.low2[u] = num2newnum[graph.low2[u]];
        graph.num[u] = newnum[u];
        graph.numrev[graph.num[u]] = u;
        graph.high[u].reverse();
    }
}
