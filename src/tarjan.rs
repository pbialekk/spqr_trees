use crate::debugging::{self, draw};
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

    for (to, eid) in adj[u].iter().map(|&eid| (edges[eid].1, eid)) {
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

            high[to].push(eid);
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

#[derive(Debug)]
struct SplitComponent {
    skeleton: Vec<usize>,
}
impl SplitComponent {
    fn new() -> Self {
        SplitComponent {
            skeleton: Vec::new(),
        }
    }
    fn add_edge(&mut self, edge: usize) {
        self.skeleton.push(edge);
    }
}

fn dfs_3(
    adj: &mut [Vec<usize>],
    edges: &mut Vec<(usize, usize)>,
    u: usize,
    parent_eid: &mut Option<usize>,
    is_dead: &mut Vec<bool>,
    tstack: &mut Vec<(usize, usize, usize)>,
    estack: &mut Vec<usize>,
    high: &mut [Vec<usize>],
    lowpt1: &[usize],
    lowpt2: &[usize],
    subsz: &[usize],
    parent: &[Option<usize>],
    deg: &mut [usize],
    split_components: &mut Vec<SplitComponent>,
    assigned_vedge: &mut Vec<usize>,
    normal_edge_count: usize,
    is_tedge: &mut Vec<bool>,
) {
    fn remove_edge(
        deg: &mut [usize],
        edges: &[(usize, usize)],
        is_dead: &mut Vec<bool>,
        eid: usize,
        assigned_vedge: &mut Vec<usize>,
        vedge: usize,
    ) {
        println!("Removing edge {}: {:?}", eid, edges[eid]);
        let (u, to) = edges[eid];
        deg[u] = deg[u].saturating_sub(1);
        deg[to] = deg[to].saturating_sub(1);
        is_dead[eid] = true;
        assigned_vedge[eid] = vedge;
    }

    fn new_vedge(
        u: usize,
        to: usize,
        adj: &mut [Vec<usize>],
        edges: &mut Vec<(usize, usize)>,
        deg: &mut [usize],
        is_dead: &mut Vec<bool>,
        assigned_vedge: &mut Vec<usize>,
        split_component: &mut SplitComponent,
        is_tedge: &mut Vec<bool>,
    ) -> usize {
        println!("Creating new virtual edge from {} to {}", u, to);
        let eid = edges.len();
        split_component.add_edge(eid);

        edges.push((u, to));
        adj[u].push(eid);

        deg[u] += 1;
        deg[to] += 1;

        is_dead.push(false);
        is_tedge.push(false); // This is a virtual edge, not a tree edge
        assigned_vedge.push(eid); // Initially, the virtual edge points to itself

        eid
    }

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
        edges: &mut Vec<(usize, usize)>,
        subsz: &[usize],
        remaining_tedges: usize,
        adj: &mut [Vec<usize>],
        deg: &mut [usize],
        is_dead: &mut Vec<bool>,
        assigned_vedge: &mut Vec<usize>,
        split_components: &mut Vec<SplitComponent>,
        parent_eid: &mut Option<usize>,
        is_tedge: &mut Vec<bool>,
    ) {
        if lowpt2[to] >= u && lowpt1[to] < u && (parent[u] != Some(0) || remaining_tedges > 0) {
            dbg!(format!("Type 1 split pair found: ({}, {})", lowpt1[to], u));
            let mut c = SplitComponent::new();
            let mut vedge = new_vedge(
                u,
                lowpt1[to],
                adj,
                edges,
                deg,
                is_dead,
                assigned_vedge,
                &mut c,
                is_tedge,
            );
            while let Some(&eid) = estack.last() {
                let (x, y) = edges[eid];

                // Check if neither x nor y is in the subtree rooted at 'to'
                let x_in_subtree = to <= x && x < to + subsz[to];
                let y_in_subtree = to <= y && y < to + subsz[to];
                if !x_in_subtree && !y_in_subtree {
                    break;
                }

                // This edge belongs to a new component, add it
                estack.pop();
                if is_dead[eid] {
                    continue; // already removedx
                }
                c.add_edge(eid);
                remove_edge(deg, edges, is_dead, eid, assigned_vedge, vedge);
            }
            split_components.push(c);

            if !estack.is_empty() {
                let &eid = estack.last().unwrap();
                let (x, y) = edges[eid];
                if x == u && y == lowpt1[to] {
                    // vedge is a multiedge, handle it
                    c = SplitComponent::new();

                    let vedge_for_c = new_vedge(
                        u,
                        lowpt1[to],
                        adj,
                        edges,
                        deg,
                        is_dead,
                        assigned_vedge,
                        &mut c,
                        is_tedge,
                    );
                    c.add_edge(vedge);
                    remove_edge(deg, edges, is_dead, vedge, assigned_vedge, vedge_for_c);
                    c.add_edge(eid);
                    remove_edge(deg, edges, is_dead, eid, assigned_vedge, vedge_for_c);
                    split_components.push(c);

                    vedge = vedge_for_c;

                    estack.pop();
                }
            }

            if Some(lowpt1[to]) != parent[u] {
                // push newly created virtual edge to the estack (it should happen in the dfs loop, but now our edges are not sorted)
                estack.push(vedge);
                dbg!(format!("Pushing new virtual edge {} to estack", vedge));
            } else {
                // our virtual edge points to parent -- we now have a multiedge, handle it as well
                let mut c = SplitComponent::new();

                let vedge_for_c = new_vedge(
                    lowpt1[to],
                    u,
                    adj,
                    edges,
                    deg,
                    is_dead,
                    assigned_vedge,
                    &mut c,
                    is_tedge,
                );
                c.add_edge(vedge);
                remove_edge(deg, edges, is_dead, vedge, assigned_vedge, vedge_for_c);
                c.add_edge(parent_eid.unwrap());
                remove_edge(
                    deg,
                    edges,
                    is_dead,
                    parent_eid.unwrap(),
                    assigned_vedge,
                    vedge_for_c,
                );
                split_components.push(c);

                vedge = vedge_for_c;

                *parent_eid = Some(vedge);
            }

            if edges[vedge].0 == u {
                let edge = &mut edges[vedge];
                swap(&mut edge.0, &mut edge.1);
                adj[lowpt1[to]].push(vedge);
                adj[u].pop();
            }
        }
    }

    fn type_2_check(
        u: usize,
        mut to: usize,
        parent: &[Option<usize>],
        estack: &mut Vec<usize>,
        tstack: &mut Vec<(usize, usize, usize)>,
        edges: &mut Vec<(usize, usize)>,
        adj: &mut [Vec<usize>],
        deg: &mut [usize],
        is_dead: &mut Vec<bool>,
        assigned_vedge: &mut Vec<usize>,
        split_components: &mut Vec<SplitComponent>,
        is_tedge: &mut Vec<bool>,
    ) {
        loop {
            let mut first_ch = 0; // first child of 'to'
            for i in 0..adj[to].len() {
                let eid = adj[to][i];
                let b_maybe = edges[adj[to][i]].1;
                if is_dead[eid] {
                    continue;
                }
                first_ch = b_maybe;
                break;
            }

            let cond_1 = u != 0 && !tstack.is_empty() && tstack.last().unwrap().1 == u;
            let cond_2 = deg[to] == 2 && first_ch > to;
            if !(cond_1 || cond_2) {
                break;
            }
            if let Some(&(h, a, b)) = tstack.last() {
                if a == u && parent[b] == Some(a) {
                    // no inner vertex exists, pop
                    tstack.pop();
                    continue;
                }
            }
            let mut eab = None;

            let mut vedge;
            let mut c = SplitComponent::new();
            if cond_2 {
                let b = first_ch;
                dbg!(format!("Type 2 split pair found: ({}, {})", u, b));
                vedge = new_vedge(
                    u,
                    b,
                    adj,
                    edges,
                    deg,
                    is_dead,
                    assigned_vedge,
                    &mut c,
                    is_tedge,
                );
                for i in 0..2 {
                    let eid = estack.pop().unwrap();
                    c.add_edge(eid);
                    remove_edge(deg, edges, is_dead, eid, assigned_vedge, vedge);
                }

                if !estack.is_empty() {
                    let &eid = estack.last().unwrap();
                    let (x, y) = edges[eid];
                    if x == u && y == b {
                        eab = Some(eid);
                        estack.pop();
                    }
                }

                split_components.push(c);
            } else {
                let (h, a, b) = tstack.pop().unwrap();
                dbg!(format!("Type 2 split pair found: ({}, {})", a, b));
                vedge = new_vedge(
                    a,
                    b,
                    adj,
                    edges,
                    deg,
                    is_dead,
                    assigned_vedge,
                    &mut c,
                    is_tedge,
                );

                while let Some(&eid) = estack.last() {
                    let (x, y) = edges[eid];

                    // Check if neither x nor y is in the subtree rooted at 'to'
                    let x_in_subtree = a <= x && x <= h;
                    let y_in_subtree = a <= y && y <= h;
                    if x_in_subtree && y_in_subtree {
                        if x == a && y == b {
                            eab = Some(eid);
                        } else {
                            c.add_edge(eid);
                            remove_edge(deg, edges, is_dead, eid, assigned_vedge, vedge);
                        }
                        estack.pop();
                    } else {
                        break;
                    }
                }

                split_components.push(c);
            }

            // handle possible multiedge
            if eab != None {
                let mut c = SplitComponent::new();
                let b = edges[vedge].1;

                c.add_edge(vedge);
                let vedge_for_c = new_vedge(
                    u,
                    b,
                    adj,
                    edges,
                    deg,
                    is_dead,
                    assigned_vedge,
                    &mut c,
                    is_tedge,
                );
                remove_edge(deg, edges, is_dead, vedge, assigned_vedge, vedge_for_c);
                c.add_edge(eab.unwrap());
                remove_edge(
                    deg,
                    edges,
                    is_dead,
                    eab.unwrap(),
                    assigned_vedge,
                    vedge_for_c,
                );
                c.add_edge(vedge_for_c);
                split_components.push(c);

                vedge = vedge_for_c;
            }
            estack.push(vedge);
            is_tedge[vedge] = true;
            to = edges[vedge].1;
        }
    }

    fn ensure_highpoints(
        u: usize,
        edges: &[(usize, usize)],
        tstack: &mut Vec<(usize, usize, usize)>,
        high: &mut [Vec<usize>],
        is_dead: &mut Vec<bool>,
    ) {
        fn get_high(
            u: usize,
            high: &mut [Vec<usize>],
            is_dead: &mut Vec<bool>,
            edges: &[(usize, usize)],
        ) -> usize {
            while let Some(&eid) = high[u].last() {
                if is_dead[eid] {
                    high[u].pop();
                    dbg!("Pop dead hp");
                } else {
                    return edges[eid].1;
                }
            }
            0 // If no high points are left, return 0
        }

        while let Some(&(h, a, b)) = tstack.last() {
            if a != u && b != u && get_high(u, high, is_dead, edges) > h {
                tstack.pop();
            } else {
                break;
            }
        }
    }

    let mut remaining_tedges = {
        adj[u]
            .iter()
            .filter(|&&eid| {
                let (from, to) = edges[eid];
                parent[to] == Some(from)
            })
            .count()
    };

    let mut i = 0;
    while i < adj[u].len() {
        let (eid, to) = {
            let eid = adj[u][i];
            (eid, edges[eid].1)
        };
        if is_dead[eid] || eid >= normal_edge_count {
            // removed edge
            i += 1;
            continue;
        }

        let starts_path = eid != adj[u][0];
        if starts_path {
            update_tstack(u, to, tstack, lowpt1, subsz, parent);
        }
        if Some(u) == parent[to] {
            remaining_tedges = remaining_tedges.saturating_sub(1);
            let mut empty_vec = Vec::new();
            dfs_3(
                adj,
                edges,
                to,
                &mut Some(eid),
                is_dead,
                if starts_path { &mut empty_vec } else { tstack },
                estack,
                high,
                lowpt1,
                lowpt2,
                subsz,
                parent,
                deg,
                split_components,
                assigned_vedge,
                normal_edge_count,
                is_tedge,
            );
            let mut e_push = eid;
            while is_dead[e_push] {
                e_push = assigned_vedge[e_push];
            }
            estack.push(e_push);
            dbg!(eid, e_push);

            type_2_check(
                u,
                to,
                parent,
                estack,
                tstack,
                edges,
                adj,
                deg,
                is_dead,
                assigned_vedge,
                split_components,
                is_tedge,
            );
            type_1_check(
                u,
                to,
                lowpt1,
                lowpt2,
                parent,
                estack,
                edges,
                subsz,
                remaining_tedges,
                adj,
                deg,
                is_dead,
                assigned_vedge,
                split_components,
                parent_eid,
                is_tedge,
            );

            ensure_highpoints(u, edges, tstack, high, is_dead);
        } else {
            // A back edge (upwards)
            if Some(to) == parent[u] {
                // A multiedge to a parent, new split component
                let mut c = SplitComponent::new();
                let e = new_vedge(
                    to,
                    u,
                    adj,
                    edges,
                    deg,
                    is_dead,
                    assigned_vedge,
                    &mut c,
                    is_tedge,
                );

                c.add_edge(eid);
                remove_edge(deg, edges, is_dead, eid, assigned_vedge, e);

                c.add_edge(parent_eid.unwrap());
                remove_edge(deg, edges, is_dead, parent_eid.unwrap(), assigned_vedge, e);

                split_components.push(c);

                parent_eid.replace(e);
                is_tedge[e] = true;
            } else {
                estack.push(eid);
            }
        }

        i += 1;
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
        let mut split_components = vec![];
        let mut is_dead = vec![false; m];
        let mut assigned_vedge = vec![0; m];
        let mut is_tedge = vec![false; m];
        for (eid, &(u, to)) in edges.iter().enumerate() {
            deg[u] += 1;
            deg[to] += 1;
            is_tedge[eid] = parent[to] == Some(u);
        }

        dfs_3(
            &mut adj,
            &mut edges,
            0,
            &mut None,
            &mut is_dead,
            &mut tstack,
            &mut estack,
            &mut high,
            &lowpt1,
            &lowpt2,
            &subsz,
            &parent,
            &mut deg,
            &mut split_components,
            &mut assigned_vedge,
            m,
            &mut is_tedge,
        );

        if !estack.is_empty() {
            let mut c = SplitComponent::new();
            while let Some(eid) = estack.pop() {
                c.add_edge(eid);
            }
            split_components.push(c);
        }

        for (i, c) in split_components.iter().enumerate() {
            let mut vertex_set = vec![];
            for &eid in &c.skeleton {
                let (u, to) = edges[eid];
                if !vertex_set.contains(&u) {
                    vertex_set.push(u);
                }
                if !vertex_set.contains(&to) {
                    vertex_set.push(to);
                }
            }
            println!("Split component {}:", i);
            println!(" Vertices: {:?}", vertex_set);
            print!(" Edges: [");
            for &eid in &c.skeleton {
                let (u, to) = edges[eid];
                if eid >= m {
                    let mut og_split_component = 0;
                    for (i, c) in split_components.iter().enumerate() {
                        if c.skeleton.contains(&eid) {
                            og_split_component = i;
                            break;
                        }
                    }
                    print!("({} <{}>), ", eid, og_split_component);
                } else {
                    print!("{}, ", eid);
                }
            }
            println!("]");
        }
    }
}
