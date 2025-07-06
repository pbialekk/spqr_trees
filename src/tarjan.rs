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

    let mut new_neighbors = vec![];

    for (eid, to) in neighbors {
        if Some(eid) == par_edge {
            continue;
        }

        if vis[to] {
            // a back edge (maybe from downwards to us!) direct from u to to and delete from 'to'
            if vis_edge[eid] {
                // already processed this edge, remove from adjacency list
                continue;
            }
            new_neighbors.push(eid);

            vis_edge[eid] = true;

            if edges[eid].0 != u {
                swap(&mut edges[eid].0, &mut edges[eid].1);
            }
            continue;
        }

        new_neighbors.push(eid);

        // A tree edge to an unvisited node, direct it from u to to
        vis_edge[eid] = true;

        if edges[eid].0 != u {
            swap(&mut edges[eid].0, &mut edges[eid].1);
        }

        // And go deeper
        dfs_0(adj, edges, vis, to, Some(eid), vis_edge);
    }

    adj[u] = new_neighbors;
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
            // A back edge (upwards), maybe to a parent (a multiedge)

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

fn dfs_3(
    adj: &[Vec<usize>],
    edges: &[(usize, usize)],
    u: usize,
    tstack: &mut Vec<(usize, usize, usize)>,
    estack: &mut Vec<usize>,
    high: &mut [Vec<usize>],
    lowpt1: &[usize],
    lowpt2: &[usize],
    subsz: &[usize],
    parent: &[Option<usize>],
    deg: &mut [usize],
) {
    let mut remaining_tree_edges = adj[u]
        .iter()
        .filter(|&&eid| {
            let to = edges[eid].1;
            Some(u) == parent[to]
        })
        .count();

    fn update_tstack(
        u: usize,
        to: usize,
        tstack: &mut Vec<(usize, usize, usize)>,
        lowpt1: &[usize],
        subsz: &[usize],
        parent: &[Option<usize>],
    ) {
        fn pop_tstack(
            cutoff: usize,
            mut max_h: usize,
            mut last_b: usize,
            tstack: &mut Vec<(usize, usize, usize)>,
        ) -> (usize, usize, usize) {
            while let Some(&(h, a, b)) = tstack.last() {
                if a > cutoff {
                    tstack.pop();
                    max_h = h.max(max_h);
                    last_b = b;
                } else {
                    break;
                }
            }

            (max_h, cutoff, last_b)
        }

        let (max_h, a, last_b) = if Some(u) == parent[to] {
            // A tree edge
            pop_tstack(lowpt1[to], to + subsz[to] - 1, u, tstack)
        } else {
            // A back edge (upwards)
            pop_tstack(to, u, u, tstack)
        };
        tstack.push((max_h, a, last_b));
    }

    fn type_1_check(
        u: usize,
        to: usize,
        lowpt1: &[usize],
        lowpt2: &[usize],
        parent: &[Option<usize>],
        estack: &mut Vec<usize>,
        edges: &[(usize, usize)],
        subsz: &[usize],
        remaining_tree_edges: usize,
    ) {
        if lowpt2[to] >= u && lowpt1[to] < u && (parent[u] != Some(0) || remaining_tree_edges > 0) {
            // TODO: a new component
            loop {
                let &eid = estack.last().unwrap();
                let (x, y) = edges[eid];

                // Check if neither x nor y is in the subtree rooted at 'to'
                let x_in_subtree = to <= x && x < to + subsz[to];
                let y_in_subtree = to <= y && y < to + subsz[to];
                if !x_in_subtree && !y_in_subtree {
                    break;
                }

                // This edge belongs to a new component, add it
                estack.pop();

                if Some(lowpt1[to]) != parent[u] {
                    // push newly created virtual edge to the estack
                } else {
                    // our virtual edge points to parent -- we now have a multiedge, handle it as well
                }
            }
        }
    }

    fn type_2_check(to: usize) {
        unimplemented!()
    }

    fn ensure_highpoints(u: usize, tstack: &mut Vec<(usize, usize, usize)>, high: &[Vec<usize>]) {
        fn get_high(u: usize, high: &[Vec<usize>]) -> usize {
            if high[u].is_empty() {
                return 0;
            }
            *high[u].last().unwrap()
        }

        while let Some(&(h, a, b)) = tstack.last() {
            if a != u && b != u && get_high(u, high) > h {
                tstack.pop();
            } else {
                break;
            }
        }
    }

    for (to, eid) in adj[u].iter().map(|&eid| (edges[eid].1, eid)) {
        let starts_path = eid != adj[u][0];
        if starts_path {
            update_tstack(u, to, tstack, lowpt1, subsz, parent);
        }
        if Some(u) == parent[to] {
            // A tree edge
            remaining_tree_edges -= 1;
            let mut empty_vec = Vec::new();
            dfs_3(
                adj,
                edges,
                to,
                if starts_path { &mut empty_vec } else { tstack },
                estack,
                high,
                lowpt1,
                lowpt2,
                subsz,
                parent,
                deg,
            );
            estack.push(eid);

            type_2_check(to);
            type_1_check(
                u,
                to,
                lowpt1,
                lowpt2,
                parent,
                estack,
                edges,
                subsz,
                remaining_tree_edges,
            );

            ensure_highpoints(u, tstack, high);
        } else {
            // A back edge (upwards)
            if Some(to) == parent[u] {
                // TODO
                // A multiedge to a parent, new split component
            } else {
                estack.push(eid);
            }
        }
    }
}

// Input: biconnected graph
pub fn cos(mut adj: Vec<Vec<usize>>, mut edges: Vec<(usize, usize)>) {
    let n = adj.len();
    let m = edges.len();

    // Step 0: direct edges from parent to son, and delete edges from son to parent
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

        for v in high.iter_mut() {
            v.reverse(); // highest point is the last in the list, so we can pop it easily
        }
    }

    println!("{}", draw(&adj, &edges, &lowpt1, &lowpt2, &parent, &subsz));

    // Step 4: finding the split components. Linked paper provides an ''easy'' conditions for a pair of vertices to be a split pair. The margin here is too narrow to explain it, so I encourage you to read https://www.inf.uni-konstanz.de/exalgo/members/mader/thesis.pdf pages 20-21. (It has a nice drawings too!) Page 13 contains the definition of a type-1/2 split pair.
    {
        let mut tstack = vec![];
        let mut estack = vec![];
        let mut deg = vec![0; n];
        dfs_3(
            &adj,
            &edges,
            0,
            &mut tstack,
            &mut estack,
            &mut high,
            &lowpt1,
            &lowpt2,
            &subsz,
            &parent,
            &mut deg,
        );
        // TODO: if estack is not empty, it's a new split component
    }
}
