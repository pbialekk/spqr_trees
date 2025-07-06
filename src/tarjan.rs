use crate::debugging::draw;
use std::mem::swap;

/// Reference: https://epubs.siam.org/doi/10.1137/0202012

fn dfs_0(
    adj: &mut [Vec<usize>],
    edges: &mut [(usize, usize)],
    vis: &mut [bool],
    u: usize,
    par_edge: Option<usize>,
    vis_edge: &mut [bool],
) {
    vis[u] = true;
    // Collect edge ids and corresponding 'to' nodes first to avoid borrowing issues
    let neighbors: Vec<(usize, usize)> = adj[u]
        .iter()
        .map(|&v| (v, edges[v].0 ^ edges[v].1 ^ u))
        .collect();

    if par_edge.is_some() {
        // If we have a parent edge, we need to remove it from the adjacency list
        adj[u].retain(|&v| v != par_edge.unwrap());
    }

    for (eid, to) in neighbors {
        if Some(eid) == par_edge {
            continue;
        }
        if vis[to] {
            // a back edge (maybe from downwards to us!) direct from u to to and delete from 'to'
            if vis_edge[eid] {
                // already processed this edge, remove from adjacency list
                // TODO: make it faster
                adj[u].retain(|&v| v != eid);
                continue;
            }
            vis_edge[eid] = true;

            if edges[eid].0 != u {
                swap(&mut edges[eid].0, &mut edges[eid].1);
            }
            continue;
        }
        // A tree edge to an unvisited node, direct it from u to to
        vis_edge[eid] = true;
        if edges[eid].0 != u {
            swap(&mut edges[eid].0, &mut edges[eid].1);
        }
        // And go deeper
        dfs_0(adj, edges, vis, to, Some(eid), vis_edge);
    }
}

fn dfs_1(
    adj: &[Vec<usize>],
    edges: &[(usize, usize)],
    u: usize,
    parent: &mut [Option<usize>],
    lowpt1: &mut [usize],
    lowpt2: &mut [usize],
    pre: &mut [usize],
    subsz: &mut [usize],
    time: &mut usize,
    high: &mut [Vec<usize>],
    second_run: bool,
) {
    // Initialize the obvious values
    pre[u] = if second_run { u } else { *time };
    lowpt1[u] = pre[u];
    lowpt2[u] = pre[u];
    subsz[u] = 1;
    *time += 1;

    for to in adj[u].iter().map(|&eid| edges[eid].1) {
        if subsz[to] == 0 {
            parent[to] = Some(u);

            dfs_1(
                adj, edges, to, parent, lowpt1, lowpt2, pre, subsz, time, high, second_run,
            );

            subsz[u] += subsz[to];

            // Update lowpt1 and lowpt2
            if lowpt1[to] < lowpt1[u] {
                lowpt2[u] = lowpt1[u].min(lowpt2[to]);
                lowpt1[u] = lowpt1[to];
            } else if lowpt1[to] == lowpt1[u] {
                lowpt2[u] = lowpt2[u].min(lowpt2[to]);
            } else {
                lowpt2[u] = lowpt2[u].min(lowpt1[to]);
            }
        } else if pre[to] < pre[u] {
            // A back edge (upwards)

            // Update lowpt1 and lowpt2
            if pre[to] < lowpt1[u] {
                lowpt2[u] = lowpt1[u];
                lowpt1[u] = pre[to];
            } else if pre[to] > lowpt1[u] {
                lowpt2[u] = lowpt2[u].min(pre[to]);
            }

            high[to].push(u);
        }
    }
}

fn dfs_2(
    adj: &[Vec<usize>],
    edges: &[(usize, usize)],
    u: usize,
    time: &mut usize,
    post: &mut [usize],
    vis: &mut [bool],
) {
    vis[u] = true;
    for to in adj[u].iter().map(|&eid| edges[eid].1) {
        if !vis[to] {
            dfs_2(adj, edges, to, time, post, vis);
        }
    }
    post[u] = *time;
    *time = time.saturating_sub(1);
}

// Input: biconnected graph
pub fn cos(mut adj: Vec<Vec<usize>>, mut edges: Vec<(usize, usize)>) {
    let n = adj.len();
    let m = edges.len();
    // TODO: Step 0: handle multiedges (just cut them off into a new split component)

    // Step 0.5: direct edges from parent to son, and delete edges from son to parent
    {
        dfs_0(
            &mut adj,
            &mut edges,
            &mut vec![false; n],
            0,
            None,
            &mut vec![false; m],
        );
    }

    // Step 1: calculate lowpt1, lowpt2, high for each vertex
    let mut parent = vec![None; n];
    let mut lowpt1 = vec![usize::MAX; n];
    let mut lowpt2 = vec![usize::MAX; n];
    let mut subsz = vec![0; n];
    let mut high = vec![vec![]; n];
    {
        let mut pre = vec![usize::MAX; n];
        let mut time = 0;
        dfs_1(
            &adj,
            &edges,
            0,
            &mut parent,
            &mut lowpt1,
            &mut lowpt2,
            &mut pre,
            &mut subsz,
            &mut time,
            &mut high,
            false,
        );
    }

    // Step 2: reorder the edges inside adjacency lists to achieve some ''good'' ordering of the edges
    {
        let phi = |edge_id: usize| -> usize {
            let (u, to) = edges[edge_id];
            let is_treeedge = || -> bool { parent[to] == Some(u) };
            if is_treeedge() {
                if lowpt2[to] < u {
                    3 * lowpt1[to]
                } else {
                    3 * lowpt1[to] + 2
                }
            } else {
                3 * to + 1
            }
        };
        let mut bucketed_edges: Vec<Vec<usize>> = vec![Vec::new(); 3 * (n - 1) + 2 + 1];
        for (eid, &(u, to)) in edges.iter().enumerate() {
            let bucket = phi(eid);
            bucketed_edges[bucket].push(eid);
        }
        let mut new_adj = vec![Vec::new(); n];
        for edge_ids in bucketed_edges.iter() {
            for &eid in edge_ids {
                let (u, to) = edges[eid];
                new_adj[u].push(eid);
            }
        }
        adj = new_adj;
    }

    // Step 3: relabel the vertices. For a vertex u, we assign the inverse of it's post-order number. This numbering has some nice properties, for example, if x, y are children of u (in this order), then u < {z | z \in Sub(y)} < x
    // Another useful property is that if x is a first descendant of u (each time we go down to a child of u, we choose the first edge in the adjacency list), then Sub(u) - Sub(x) = {y | u <= y < x}

    let mut post = vec![usize::MAX; n]; // 0 becomes post[0], ...
    {
        let mut time = n - 1;
        dfs_2(&adj, &edges, 0, &mut time, &mut post, &mut vec![false; n]);

        // We map v to post[v]
        for (a, b) in edges.iter_mut() {
            *a = post[*a];
            *b = post[*b];
        }
        let mut new_adj = vec![Vec::new(); n];
        for (eid, &(a, b)) in edges.iter().enumerate() {
            new_adj[a].push(eid);
        }
        adj = new_adj;

        // Step 3.5: relabel the parent, lowpt1, lowpt2. Just run dfs_1 again with the new adjacency list
        let mut new_parent = vec![None; n];
        let mut new_lowpt1 = vec![usize::MAX; n];
        let mut new_lowpt2 = vec![usize::MAX; n];
        let mut new_subsz = vec![0; n];
        let mut new_high = vec![vec![]; n];
        {
            let mut pre = vec![usize::MAX; n];
            let mut time = 0;
            dfs_1(
                &adj,
                &edges,
                0,
                &mut new_parent,
                &mut new_lowpt1,
                &mut new_lowpt2,
                &mut pre,
                &mut new_subsz,
                &mut time,
                &mut new_high,
                true,
            );
            parent = new_parent;
            lowpt1 = new_lowpt1;
            lowpt2 = new_lowpt2;
            subsz = new_subsz; // should be the same as before.
            high = new_high;
        }
    }

    println!("{}", draw(&adj, &edges, &lowpt1, &lowpt2, &parent, &subsz));
}
