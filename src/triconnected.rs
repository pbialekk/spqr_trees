use std::mem::{self};

use petgraph::visit::EdgeRef;

use crate::{UnGraph, block_cut::get_block_cut_tree, debugging};

/// Reference: https://epubs.siam.org/doi/10.1137/0202012
// TODO: describe general idea

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    Tree,
    Back,
    Killed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    P, // bond
    S, // triangle
    R, // triconnected
}

#[derive(Debug, Clone)]
pub struct Component {
    pub edges: Vec<usize>,
    pub component_type: Option<ComponentType>,
}

impl Component {
    pub fn new(component_type: Option<ComponentType>) -> Self {
        Self {
            edges: Vec::new(),
            component_type,
        }
    }

    pub fn push_edge(&mut self, edge: usize) -> &mut Self {
        self.edges.push(edge);
        self
    }

    pub fn commit(&mut self, split_components: &mut Vec<Component>) {
        if self.component_type.is_none() {
            self.component_type = Some(if self.edges.len() >= 4 {
                ComponentType::R
            } else {
                ComponentType::S
            });
        }

        split_components.push(self.clone());
    }
}

// holds num[v], par[v], low1[v], low2[v], sub[v], deg[v]
#[derive(Debug, Clone)]
struct GraphInternal {
    adj: Vec<Vec<usize>>, // adjacency list, edges are stored as indices in `edges`
    edges: Vec<(usize, usize)>, // edges in the form (source, target)
    edge_type: Vec<Option<EdgeType>>, // edge type, None means not visited yet

    num: Vec<usize>,

    par_edge: Vec<Option<usize>>, // edge id of the parent edge in DFS tree
    par: Vec<Option<usize>>,
    low1: Vec<usize>,
    low2: Vec<usize>,
    sub: Vec<usize>,
    deg: Vec<usize>,

    high: Vec<Vec<usize>>, // pathfinder starts here
    numrev: Vec<usize>,    // reverse mapping from num to original vertex
    starts_path: Vec<bool>,
}
impl GraphInternal {
    fn new(n: usize, m: usize) -> Self {
        Self {
            adj: vec![Vec::new(); n], // adjacency list, edges are stored as indices in `edges`
            edges: Vec::new(),        // edges in the form (source, target)
            edge_type: Vec::new(),    // edge type, None means not visited yet

            num: vec![usize::MAX; n],

            par_edge: vec![None; n], // edge id of the parent edge in DFS tree
            par: vec![None; n],
            low1: vec![0; n],
            low2: vec![0; n],
            sub: vec![0; n],
            deg: vec![0; n],

            high: vec![Vec::new(); n], // pathfinder starts here
            numrev: vec![0; n],        // reverse mapping from num to original vertex
            starts_path: vec![false; m],
        }
    }
    fn new_edge(&mut self, s: usize, t: usize, put_type: Option<EdgeType>) -> usize {
        let eid = self.edges.len();

        if cfg!(debug_assertions) {
            println!("[{}] Adding edge ({}, {})", eid, s, t);
        }

        self.edges.push((s, t));
        self.edge_type.push(put_type);
        self.adj[s].push(eid);
        self.starts_path.push(false);
        self.deg[s] += 1;
        self.deg[t] += 1;

        eid
    }
    fn kill_edge(&mut self, eid: usize) {
        if cfg!(debug_assertions) {
            println!(
                "Killing edge {}: ({}, {})",
                eid, self.edges[eid].0, self.edges[eid].1
            );
        }

        assert!(
            self.edge_type[eid] != Some(EdgeType::Killed),
            "Edge {} is already dead",
            eid
        );

        self.edge_type[eid] = Some(EdgeType::Killed);
        let (s, t) = self.edges[eid];
        self.deg[s] -= 1;
        self.deg[t] -= 1;
    }
    fn make_tedge(&mut self, eid: usize) {
        if cfg!(debug_assertions) {
            println!(
                "Making edge {} a tree edge: ({}, {})",
                eid, self.edges[eid].0, self.edges[eid].1
            );
        }

        self.edge_type[eid] = Some(EdgeType::Tree);
        let (s, t) = self.edges[eid];

        self.par_edge[t] = Some(eid);
        self.par[t] = Some(s);
    }
    fn make_bedge(&mut self, eid: usize) {
        if cfg!(debug_assertions) {
            println!(
                "Making edge {} a back edge: ({}, {})",
                eid, self.edges[eid].0, self.edges[eid].1
            );
        }

        self.edge_type[eid] = Some(EdgeType::Back);
        let (s, t) = self.edges[eid];

        if self.get_high(s) < self.num[s] {
            self.high[t].push(eid);
        }
    }
    fn get_other(&self, eid: usize, u: usize) -> usize {
        let (s, t) = self.edges[eid];
        if s == u { t } else { s }
    }
    fn first_alive(&self, root: usize, u: usize) -> usize {
        if u == root {
            return usize::MAX; // every tree edge from root starts a new path
        }
        for &eid in &self.adj[u] {
            if self.edge_type[eid] == Some(EdgeType::Killed) {
                continue;
            }
            return self.edges[eid].1;
        }
        return usize::MAX;
    }
    fn get_high(&mut self, u: usize) -> usize {
        while let Some(&eid) = self.high[u].last() {
            if self.edge_type[eid] == Some(EdgeType::Killed) {
                self.high[u].pop();

                if cfg!(debug_assertions) {
                    println!("Removing killed edge {} from highpoint of {}", eid, u);
                }
            } else {
                return self.num[self.get_other(eid, u)];
            }
        }
        0
    }
}

fn find_split(
    root: usize,
    u: usize,
    vedges_cutoff: usize,
    graph: &mut GraphInternal,
    estack: &mut Vec<usize>,
    tstack: &mut Vec<(usize, usize, usize)>,
    split_components: &mut Vec<Component>,
) {
    fn update_tstack(
        u: usize,
        to: usize,
        eid: usize,
        tstack: &mut Vec<(usize, usize, usize)>,
        graph: &GraphInternal,
    ) {
        fn pop_tstack(
            cutoff: usize,
            mut max_h: usize,
            mut last_b: usize,
            tstack: &mut Vec<(usize, usize, usize)>,
        ) -> (usize, usize, usize) {
            while let Some(&(h, a, b)) = tstack.last() {
                if a > cutoff {
                    if cfg!(debug_assertions) {
                        println!("Popping tstack: ({}, {}, {})", h, a, b);
                    }
                    tstack.pop();
                    max_h = h.max(max_h);
                    last_b = b;
                } else {
                    break;
                }
            }

            (max_h, cutoff, last_b)
        }

        let (max_h, a, last_b) = if graph.edge_type[eid] == Some(EdgeType::Tree) {
            pop_tstack(
                graph.low1[to],
                graph.num[to] + graph.sub[to] - 1,
                graph.num[u],
                tstack,
            )
        } else {
            pop_tstack(graph.num[to], graph.num[u], graph.num[u], tstack)
        };

        tstack.push((max_h, a, last_b));

        if cfg!(debug_assertions) {
            println!(
                "Pushing to tstack: ({}, {}, {}) for edge {} from {} to {}",
                max_h, a, last_b, eid, u, to
            );
        }
    }

    fn ensure_highpoint(
        u: usize,
        tstack: &mut Vec<(usize, usize, usize)>,
        graph: &mut GraphInternal,
    ) {
        let u_high = graph.get_high(u);

        while let Some(&(h, a, b)) = tstack.last() {
            if a != graph.num[u] && b != graph.num[u] && u_high > h {
                if cfg!(debug_assertions) {
                    println!(
                        "Popping tstack due to ensure_highpoint: ({}, {}, {})",
                        h, a, b
                    );
                }

                tstack.pop();
            } else {
                break;
            }
        }
    }

    fn check_type_2(
        root: usize,
        u: usize,
        mut to: usize,
        tstack: &mut Vec<(usize, usize, usize)>,
        estack: &mut Vec<usize>,
        graph: &mut GraphInternal,
        split_components: &mut Vec<Component>,
    ) {
        if graph.num[u] == root {
            return;
        }

        loop {
            let (h, a, b) = if let Some(&last) = tstack.last() {
                last
            } else {
                (0, usize::MAX, 0)
            };

            let cond_1 = a == graph.num[u];
            let cond_2 =
                graph.deg[to] == 2 && graph.num[graph.first_alive(root, to)] > graph.num[to];

            if !(cond_1 || cond_2) {
                break;
            }
            if a == graph.num[u] && graph.par[graph.numrev[b]] == Some(u) {
                if cfg!(debug_assertions) {
                    println!(
                        "Popping {} {} from tstack: no inner vertex exists",
                        graph.numrev[a], graph.numrev[b]
                    );
                }

                tstack.pop();
                continue;
            }

            let mut eab = usize::MAX;
            let mut evirt = usize::MAX;
            if cond_2 {
                to = graph.first_alive(root, to);

                if cfg!(debug_assertions) {
                    println!("Type 2 pair found (easy one) ({}, {})", u, to);
                }

                let mut component = Component::new(Some(ComponentType::S));

                for _ in 0..2 {
                    let e = estack.pop().unwrap();
                    graph.kill_edge(e);
                    component.push_edge(e);
                }

                evirt = graph.new_edge(u, to, None);
                component.push_edge(evirt);

                component.commit(split_components);

                if let Some(&e) = estack.last() {
                    if graph.edges[e] == (to, u) {
                        estack.pop();
                        eab = e;
                    }
                }
            } else {
                to = graph.numrev[b];
                if cfg!(debug_assertions) {
                    println!("Type 2 pair found (hard one) ({}, {})", u, to);
                }

                tstack.pop();
                let mut component = Component::new(None);
                loop {
                    if let Some(&e) = estack.last() {
                        let (x, y) = graph.edges[e];

                        let x_in_subtree = graph.num[u] <= graph.num[x] && graph.num[x] <= h;
                        let y_in_subtree = graph.num[u] <= graph.num[y] && graph.num[y] <= h;
                        if !(x_in_subtree && y_in_subtree) {
                            break;
                        }

                        estack.pop();

                        if [
                            graph.num[x].min(graph.num[y]),
                            graph.num[x].max(graph.num[y]),
                        ] == [graph.num[u], graph.num[to]]
                        {
                            eab = e;
                        } else {
                            graph.kill_edge(e);
                            component.push_edge(e);
                        }
                    } else {
                        break;
                    }
                }

                evirt = graph.new_edge(u, to, None);
                component.push_edge(evirt);
                component.commit(split_components);
            }

            if eab != usize::MAX {
                let mut component = Component::new(Some(ComponentType::P));
                component.push_edge(eab);
                graph.kill_edge(eab);

                component.push_edge(evirt);
                graph.kill_edge(evirt);

                evirt = graph.new_edge(u, to, None);
                component.push_edge(evirt);

                component.commit(split_components);
            }

            estack.push(evirt);
            graph.make_tedge(evirt);
        }
    }
    fn check_type_1(
        root: usize,
        u: usize,
        to: usize,
        estack: &mut Vec<usize>,
        graph: &mut GraphInternal,
        split_components: &mut Vec<Component>,
        t_edges_left: usize,
    ) {
        if graph.low2[to] >= graph.num[u]
            && graph.low1[to] < graph.num[u]
            && (Some(root) != graph.par[u] || t_edges_left != 0)
        {
            if cfg!(debug_assertions) {
                println!(
                    "Type 1 pair found ({}, {})",
                    graph.numrev[graph.low1[to]], u
                );
            }

            let mut component = Component::new(None);
            while let Some(&eid) = estack.last() {
                let (x, y) = graph.edges[eid];
                let x_in_subtree =
                    graph.num[to] <= graph.num[x] && graph.num[x] < graph.num[to] + graph.sub[to];
                let y_in_subtree =
                    graph.num[to] <= graph.num[y] && graph.num[y] < graph.num[to] + graph.sub[to];

                if !(x_in_subtree || y_in_subtree) {
                    break;
                }

                estack.pop();

                component.push_edge(eid);
                graph.kill_edge(eid);
            }

            let mut evirt = graph.new_edge(u, graph.numrev[graph.low1[to]], None);
            component.push_edge(evirt);

            component.commit(split_components);

            if let Some(&eid) = estack.last() {
                let (x, y) = graph.edges[eid];
                if (x == u && y == graph.numrev[graph.low1[to]])
                    || (y == u && x == graph.numrev[graph.low1[to]])
                {
                    estack.pop();
                    let mut component = Component::new(Some(ComponentType::P));

                    component.push_edge(eid);
                    graph.kill_edge(eid);

                    component.push_edge(evirt);
                    graph.kill_edge(evirt);

                    evirt = graph.new_edge(u, graph.numrev[graph.low1[to]], None);
                    component.push_edge(evirt);

                    component.commit(split_components);
                }
            }

            if Some(graph.numrev[graph.low1[to]]) != graph.par[u] {
                estack.push(evirt);

                graph.make_bedge(evirt);
            } else {
                let parent_edge = graph.par_edge[u].unwrap();

                let mut component = Component::new(Some(ComponentType::P));

                component.push_edge(parent_edge);
                graph.kill_edge(parent_edge);

                component.push_edge(evirt);
                graph.kill_edge(evirt);

                evirt = graph.new_edge(graph.par[u].unwrap(), u, None);
                component.push_edge(evirt);

                component.commit(split_components);

                graph.make_tedge(evirt);
                graph.par_edge[u] = Some(evirt);
            }
        }
    }

    let mut adjacent_tedges = graph.adj[u]
        .iter()
        .filter(|&eid| graph.edge_type[*eid] == Some(EdgeType::Tree))
        .count();

    let mut i = 0;
    while i < graph.adj[u].len() {
        let eid = graph.adj[u][i];
        if graph.edge_type[eid] == Some(EdgeType::Killed) || eid >= vedges_cutoff {
            // we don't care about killer nor virtual edges here
            i += 1;
            continue;
        }

        let to = graph.get_other(eid, u);
        if graph.starts_path[eid] {
            update_tstack(u, to, eid, tstack, graph);
        }

        if graph.edge_type[eid] == Some(EdgeType::Tree) {
            let mut new_tstack = vec![];
            find_split(
                root,
                to,
                vedges_cutoff,
                graph,
                estack,
                if graph.starts_path[eid] {
                    &mut new_tstack
                } else {
                    tstack
                },
                split_components,
            );
            adjacent_tedges -= 1;

            let push_eid = graph.par_edge[to].unwrap(); // eid could be killed by the multiple edge case in check_type_x
            estack.push(push_eid);

            check_type_2(
                root,
                u,
                to,
                if graph.starts_path[eid] {
                    &mut new_tstack
                } else {
                    tstack
                },
                estack,
                graph,
                split_components,
            );
            check_type_1(
                root,
                u,
                to,
                estack,
                graph,
                split_components,
                adjacent_tedges,
            );

            ensure_highpoint(u, tstack, graph);
        } else {
            estack.push(eid);
        }

        i += 1;
    }
}

pub fn get_triconnected_components(in_graph: &UnGraph) -> (Vec<Component>, Vec<(usize, usize)>) {
    let n = in_graph.node_count();
    let m = in_graph.edge_count();
    let root = 0;

    let mut split_components = Vec::new();

    if cfg!(debug_assertions) {
        println!("{} nodes, {} edges", n, m);
    }

    assert!(get_block_cut_tree(&in_graph).block_count == 1);
    assert!(n >= 2);

    if n == 2 {
        let mut c = Component::new(Some(ComponentType::P));
        let mut edges = Vec::new();
        for i in in_graph.edge_references() {
            let (s, t) = (i.source().index(), i.target().index());
            edges.push((s, t));
            c.push_edge(i.id().index());
        }

        if m >= 3 {
            return (vec![c], edges);
        } else {
            return (vec![], edges);
        }
    }

    let mut graph = GraphInternal::new(n, m);

    // construct edges and adj arrays
    {
        for i in in_graph.edge_references() {
            let (mut s, mut t) = (i.source().index(), i.target().index());
            if s > t {
                mem::swap(&mut s, &mut t);
            }

            graph.new_edge(s, t, None);
        }
    }

    // self-explanatory

    {
        if cfg!(debug_assertions) {
            println!("Edges before sorting and handling duplicates:");
            for (eid, edge) in graph.edges.iter().enumerate() {
                if graph.edge_type[eid] == Some(EdgeType::Killed) {
                    continue; // skip killed edges
                }

                println!("{}:\t ({}, {})", eid, edge.0, edge.1);
            }
        }

        fn sort_pairs_upperbounded(edges: &mut Vec<(usize, usize)>, upper_bound: usize) {
            // TODO: implement a bucketsort here
            edges.sort();
        }

        sort_pairs_upperbounded(&mut graph.edges, n);

        fn handle_duplicate_edges(
            graph: &mut GraphInternal,
            split_components: &mut Vec<Component>,
        ) {
            graph.adj = vec![Vec::new(); graph.adj.len()]; // reset adjacency list

            let mut i = 0;
            let len = graph.edges.len();

            while i < len {
                let (s, t) = graph.edges[i];
                if s == t {
                    // self-loop, we don't care about them
                    i += 1;
                    continue;
                }

                if i + 1 < len && graph.edges[i] == graph.edges[i + 1] {
                    let mut component = Component::new(Some(ComponentType::P));

                    let (s, t) = graph.edges[i];
                    let eid = graph.new_edge(s, t, None);
                    component.push_edge(eid);
                    graph.adj[t].push(eid); // add t->s edge as well, since we are not rooted yet

                    component.push_edge(i);
                    graph.kill_edge(i);

                    while i + 1 < len && graph.edges[i + 1] == graph.edges[i] {
                        i += 1;

                        component.push_edge(i);
                        graph.kill_edge(i);
                    }

                    split_components.push(component);
                } else {
                    graph.adj[s].push(i);
                    graph.adj[t].push(i); // add both directions, since we are not rooted yet
                }

                i += 1;
            }
        }

        handle_duplicate_edges(&mut graph, &mut split_components);

        if cfg!(debug_assertions) {
            println!("Edges after sorting and handling duplicates:");
            for (eid, edge) in graph.edges.iter().enumerate() {
                if graph.edge_type[eid] == Some(EdgeType::Killed) {
                    continue; // skip killed edges
                }

                println!("{}:\t ({}, {})", eid, edge.0, edge.1);
            }
        }
    }
    let m = graph.edges.len();

    // first dfs, computes num, low1, low2, sub, par, deg, edge_type and fixes the edges' direction
    {
        let mut time = 0;

        fn dfs_precomp(u: usize, time: &mut usize, graph: &mut GraphInternal) {
            graph.num[u] = *time;
            graph.low1[u] = *time;
            graph.low2[u] = *time;
            graph.sub[u] = 1;
            *time += 1;

            let neighbors = graph.adj[u].clone(); // borrow checker doesn't like mutable borrow below

            for &eid in &neighbors {
                if graph.edge_type[eid] == Some(EdgeType::Killed) {
                    continue; // skip killed edges
                }

                let to = graph.get_other(eid, u);

                if graph.edge_type[eid].is_some() {
                    continue; // already visited 
                }

                if graph.num[to] == usize::MAX {
                    // tree edge
                    graph.par_edge[to] = Some(eid);
                    graph.par[to] = Some(u);
                    graph.edge_type[eid] = Some(EdgeType::Tree);

                    dfs_precomp(to, time, graph);

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

        dfs_precomp(root, &mut time, &mut graph);

        // now that for each edge we know its type, we can assure that edges in `edges` always point from source to target
        for (eid, edge) in graph.edges.iter_mut().enumerate() {
            if graph.edge_type[eid] == Some(EdgeType::Killed) {
                continue; // skip killed edges
            }

            let (s, t) = (edge.0, edge.1);
            if (graph.edge_type[eid] == Some(EdgeType::Back) && graph.num[s] < graph.num[t])
                || (graph.edge_type[eid] == Some(EdgeType::Tree) && graph.num[s] > graph.num[t])
            {
                mem::swap(&mut edge.0, &mut edge.1);
            }
        }

        if cfg!(debug_assertions) {
            println!("DFS1 finished");
            println!("u\t num\t low1\t low2\t sub\t deg");
            for u in 0..n {
                println!(
                    "{}\t {}\t {}\t {}\t {}\t {}",
                    u, graph.num[u], graph.low1[u], graph.low2[u], graph.sub[u], graph.deg[u]
                );
            }

            println!("eid\t (s, t)\t edge_type");
            for eid in 0..graph.edges.len() {
                let (s, t) = graph.edges[eid];
                println!("{}:\t ({}, {})\t{:?}", eid, s, t, graph.edge_type[eid]);
            }
        }
    }

    // compute acceptable adjacency list structure
    {
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

        if cfg!(debug_assertions) {
            println!("Adjacency list before sorting:");
            for (u, edges) in graph.adj.iter().enumerate() {
                println!("{}: {:?}", u, edges);
            }
        }

        graph.adj = new_adj;

        if cfg!(debug_assertions) {
            println!("Adjacency list after sorting:");
            for (u, edges) in graph.adj.iter().enumerate() {
                println!("{}: {:?}", u, edges);
            }
        }
    }

    // pathfinder part: calculate high(v), newnum(v), starts_path(e)
    {
        let mut newnum = vec![0; n];
        let mut time = n - 1;

        fn pathfinder_dfs(
            root: usize,
            u: usize,
            graph: &mut GraphInternal,
            newnum: &mut Vec<usize>,
            time: &mut usize,
        ) {
            let first_to = graph.first_alive(root, u);

            let neighbors = graph.adj[u].clone(); // borrow checker doesn't like mutable borrow below

            for &eid in neighbors.iter() {
                let to = graph.get_other(eid, u);
                // no killed edges here

                if to != first_to {
                    graph.starts_path[eid] = true;
                }

                if graph.edge_type[eid] == Some(EdgeType::Tree) {
                    pathfinder_dfs(root, to, graph, newnum, time);
                } else {
                    // always a back edge
                    graph.high[to].push(eid);
                }
            }

            newnum[u] = *time;
            *time = time.saturating_sub(1);
        }

        pathfinder_dfs(root, root, &mut graph, &mut newnum, &mut time);

        // now we need to renumber the vertices from num(v) to newnum(v)
        let mut num2newnum = vec![0; n];
        for u in 0..n {
            num2newnum[graph.num[u]] = newnum[u];
        }

        for u in 0..n {
            graph.low1[u] = num2newnum[graph.low1[u]];
            graph.low2[u] = num2newnum[graph.low2[u]];
            graph.num[u] = newnum[u];
            graph.numrev[graph.num[u]] = u;
            graph.high[u].reverse();
        }

        if cfg!(debug_assertions) {
            println!("Pathfinder finished");
            println!("u\t num\t low1\t low2\t sub\t deg\t high");
            for u in 0..n {
                println!(
                    "{}\t {}\t {}\t {}\t {}\t {}\t {:?}",
                    u,
                    graph.num[u],
                    graph.low1[u],
                    graph.low2[u],
                    graph.sub[u],
                    graph.deg[u],
                    graph.high[u],
                );
            }

            println!("eid\t (s, t)\t edge_type\t starts_path");
            for eid in 0..graph.edges.len() {
                let (s, t) = graph.edges[eid];
                println!(
                    "{}:\t ({}, {})\t{:?}\t {}",
                    eid, s, t, graph.edge_type[eid], graph.starts_path[eid]
                );
            }
        }
    }

    if cfg!(debug_assertions) {
        let dot_output = debugging::draw(
            &graph.adj,
            &graph.edges,
            &graph.num,
            &graph.high,
            &graph.edge_type,
            &graph.starts_path,
            &graph.low1,
            &graph.low2,
            &graph.par,
            &graph.sub,
        );

        std::fs::write("pre_last.dot", dot_output).expect("Unable to write to pre_last.dot");
        std::process::Command::new("dot")
            .args(&["pre_last.dot", "-Tpng", "-o", "pre_last.png"])
            .status()
            .expect("failed to execute dot");
    }

    // detect split_components
    {
        let mut estack = Vec::new();
        let mut tstack = Vec::new();
        find_split(
            root,
            root,
            graph.edges.len(),
            &mut graph,
            &mut estack,
            &mut tstack,
            &mut split_components,
        );

        let mut component = Component::new(None);
        while let Some(eid) = estack.pop() {
            component.push_edge(eid);
            graph.edge_type[eid] = Some(EdgeType::Killed);
        }
        component.commit(&mut split_components);

        if cfg!(debug_assertions) {
            println!("eid\t (s, t)\t edge_type\t starts_path");
            for eid in 0..graph.edges.len() {
                let (s, t) = graph.edges[eid];
                println!(
                    "{}:\t ({}, {})\t{:?}\t {}",
                    eid, s, t, graph.edge_type[eid], graph.starts_path[eid]
                );
            }

            println!("Split components found:");
            for (i, component) in split_components.iter().enumerate() {
                println!(
                    "Component {}: type: {:?}, edges: {:?}",
                    i, component.component_type, component.edges
                );
            }
        }

        // merge S and P nodes
        {
            let mut final_components = vec![];

            let mut pos = vec![0; graph.edges.len()];
            let XD_split_components = split_components.clone();

            for (i, component) in split_components.iter().enumerate() {
                for &eid in component.edges.iter() {
                    pos[eid] = i; // vedges occur twice
                }
            }

            // TODO: make it faster, maybe use linked lists? xd
            let mut dead = vec![false; split_components.len()];
            for (i, component) in split_components.iter().enumerate() {
                if component.component_type == Some(ComponentType::R) {
                    final_components.push(component.clone());
                    continue; // tcc 
                }
                if dead[i] {
                    continue; // already dead
                }

                let mut new_component = component.clone();
                let mut j = 0;

                while j < new_component.edges.len() {
                    let eid = new_component.edges[j];
                    if pos[eid] != i {
                        let guy = pos[eid];

                        let guy_component = &XD_split_components[guy];

                        if !dead[guy] && guy_component.component_type == component.component_type {
                            dead[guy] = true;
                            new_component
                                .edges
                                .extend(guy_component.edges.iter().filter(|&&e| e != eid));
                            // graph.kill_edge(eid);
                        }
                    }

                    j += 1;
                }

                if !new_component.edges.is_empty() {
                    final_components.push(new_component);
                }
            }

            split_components = final_components;

            if cfg!(debug_assertions) {
                println!("Final components after merging S and P nodes:");
                for (i, component) in split_components.iter().enumerate() {
                    println!(
                        "Component {}: type: {:?}, edges: {:?}",
                        i, component.component_type, component.edges
                    );
                }
            }
        }

        if cfg!(debug_assertions) && false {
            let dot_output = debugging::draw_components(&split_components, n, &graph.edges);

            std::fs::write("post_last.dot", dot_output).expect("Unable to write to post_last.dot");
            std::process::Command::new("dot")
                .args(&["post_last.dot", "-Tpng", "-o", "post_last.png"])
                .status()
                .expect("failed to execute dot");
        }

        (split_components, graph.edges)
    }
}

#[cfg(test)]
mod tests {
    use petgraph::visit::{IntoNodeReferences, NodeIndexable};

    use crate::{EdgeLabel, block_cut::get_block_cut_tree};

    use super::*;

    fn are_triconnected_brute(in_graph: &UnGraph) -> Vec<Vec<bool>> {
        let n = in_graph.node_references().count();
        let mut res: Vec<Vec<bool>> = vec![vec![false; n]; n];
        let mut cap = vec![vec![0; n * 2]; n * 2]; // indices from 0 to n-1 are 'ins', rest are 'outs'

        for (u, v) in in_graph
            .edge_references()
            .map(|e| (e.source().index(), e.target().index()))
        {
            cap[u + n][v] += 1;
            cap[v + n][u] += 1;
        }
        for u in 0..n {
            cap[u][u + n] += 1; // ins to outs
        }

        fn is_3_conn(s: usize, t: usize, cap: &Vec<Vec<usize>>) -> bool {
            let mut cap = cap.clone();
            let mut vis = vec![false; cap.len()];
            fn dfs(u: usize, t: usize, cap: &mut Vec<Vec<usize>>, vis: &mut Vec<bool>) -> bool {
                vis[u] = true;
                if u == t {
                    return true;
                }
                for v in 0..cap.len() {
                    if !vis[v] && cap[u][v] > 0 {
                        if dfs(v, t, cap, vis) {
                            cap[u][v] -= 1;
                            cap[v][u] += 1;
                            return true;
                        }
                    }
                }
                false
            }
            for i in 0..3 {
                if !dfs(s + cap.len() / 2, t, &mut cap, &mut vis) {
                    return false;
                }
                vis.fill(false);
            }
            true
        }

        for u in 0..n {
            for v in 0..n {
                if u == v {
                    continue;
                }
                res[u][v] = is_3_conn(u, v, &cap);
            }
        }

        res
    }
    fn random_biconnected_graph(n: usize, m: usize, seed: usize) -> UnGraph {
        use rand::Rng;
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let mut rng = StdRng::seed_from_u64(seed as u64);
        let mut graph = UnGraph::new_undirected();

        for i in 0..n {
            graph.add_node(i.try_into().unwrap());
            if i > 0 {
                let j = rng.random_range(0..i);
                graph.add_edge(graph.from_index(i), graph.from_index(j), EdgeLabel::Real);
            }
        }

        for _ in n - 1..m {
            let s = rng.random_range(0..n);
            let t = rng.random_range(0..n);
            graph.add_edge(graph.from_index(s), graph.from_index(t), EdgeLabel::Real);
        }

        let bct = get_block_cut_tree(&graph);

        bct.blocks[0].clone()
    }

    fn answer_fast(
        n: usize,
        m: usize,
        split_components: &Vec<Component>,
        edges: &Vec<(usize, usize)>,
    ) -> Vec<Vec<bool>> {
        if n == 2 && m <= 2 {
            return vec![vec![false, false], vec![false, false]];
        }
        let mut res = vec![vec![false; n]; n];

        for c in split_components {
            if c.component_type == Some(ComponentType::S) {
                // not triconnected
                continue;
            }

            let mut vertex_set = Vec::new();
            for e in c.edges.iter() {
                let (u, v) = edges[*e];
                vertex_set.push(u);
                vertex_set.push(v);
            }
            vertex_set.sort();
            vertex_set.dedup();

            for &x in &vertex_set {
                for &y in &vertex_set {
                    if x != y {
                        res[x][y] = true;
                    }
                }
            }
        }

        res
    }
    fn is_splitpair(in_graph: &UnGraph, s: usize, t: usize) -> bool {
        let n = in_graph.node_references().count();
        let mut vis = vec![false; n];
        fn dfs(u: usize, in_graph: &UnGraph, vis: &mut Vec<bool>) {
            vis[u] = true;
            for v in in_graph.neighbors(in_graph.from_index(u)) {
                if !vis[v.index()] {
                    dfs(v.index(), in_graph, vis);
                }
            }
        }

        vis[s] = true;
        vis[t] = true;

        for i in 0..n {
            if i == s || i == t {
                continue;
            }
            dfs(i, in_graph, &mut vis);
            break;
        }

        let mut direct_cnt = 0;
        for v in in_graph.neighbors(in_graph.from_index(s)) {
            if v.index() == t {
                direct_cnt += 1;
            }
        }

        vis.iter().any(|&v| !v) || direct_cnt > 1
    }
    fn verify_components(
        in_graph: &UnGraph,
        split_components: &Vec<Component>,
        edges: &Vec<(usize, usize)>,
    ) {
        let n = in_graph.node_references().count();
        let m = edges.len();

        let mut edges_occs = vec![0; m];
        for c in split_components {
            for &eid in &c.edges {
                edges_occs[eid] += 1;
            }

            if c.component_type == Some(ComponentType::P) {
                let mut nodes = vec![];
                for &eid in &c.edges {
                    let (s, t) = edges[eid];
                    nodes.push(s);
                    nodes.push(t);
                }
                nodes.sort();
                nodes.dedup();

                assert!(nodes.len() == 2);
            } else if c.component_type == Some(ComponentType::S) {
                let mut nodes = vec![];
                for &eid in &c.edges {
                    let (s, t) = edges[eid];
                    nodes.push(s);
                    nodes.push(t);
                }
                nodes.sort();
                nodes.dedup();

                assert!(nodes.len() >= 3);
                assert!(c.edges.len() == nodes.len());

                let mut deg = vec![0; n];
                for &eid in &c.edges {
                    let (s, t) = edges[eid];
                    deg[s] += 1;
                    deg[t] += 1;
                }

                assert!(deg.iter().all(|&d| d == 0 || d == 2));
            } else if c.component_type == Some(ComponentType::R) {
                let mut nodes = vec![];
                for &eid in &c.edges {
                    let (s, t) = edges[eid];
                    nodes.push(s);
                    nodes.push(t);
                }
                nodes.sort();
                nodes.dedup();

                assert!(nodes.len() >= 4);
            } else {
                panic!();
            }
        }

        assert!(*edges_occs.iter().max().unwrap() <= 2);

        // if an edge occurs twice, then it's a vedge -- thus, a split pair.
        for (eid, cnt) in edges_occs.iter().enumerate() {
            if *cnt == 0 {
                continue; // edge is not in any component
            }

            let (s, t) = edges[eid];
            if *cnt == 2 {
                assert!(is_splitpair(in_graph, s, t));
            }
        }
    }

    // Only run this test in release mode (i.e., when !(debug_assertions))
    #[cfg(all(test, not(debug_assertions)))]
    #[test]
    fn test_triconnected_components() {
        for i in 0..1000 {
            println!("test_triconnected_components() it: {}", i);

            let n = 2 + i / 10;
            let m: usize = 1 + i;

            let in_graph = random_biconnected_graph(n, m, i);

            let (split_components, edges) = get_triconnected_components(&in_graph);
            verify_components(&in_graph, &split_components, &edges);

            let n = in_graph.node_references().count();
            let m = in_graph.edge_references().count();

            let brute_mat = are_triconnected_brute(&in_graph);
            let fast_mat = answer_fast(n, m, &split_components, &edges);

            assert_eq!(brute_mat, fast_mat);
        }
    }

    #[test]
    fn test_triconnected_components_light() {
        for i in 0..100 {
            println!("test_triconnected_components_light() it: {}", i);

            let n = 2 + i / 10;
            let m: usize = 1 + i;

            let in_graph = random_biconnected_graph(n, m, i);

            dbg!(&in_graph);

            let (split_components, edges) = get_triconnected_components(&in_graph);
            verify_components(&in_graph, &split_components, &edges);

            let n = in_graph.node_references().count();
            let m = in_graph.edge_references().count();

            let brute_mat = are_triconnected_brute(&in_graph);
            let fast_mat = answer_fast(n, m, &split_components, &edges);

            assert_eq!(brute_mat, fast_mat);
        }
    }
}
