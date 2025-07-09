use std::mem::{self};

use petgraph::visit::EdgeRef;

use crate::{UnGraph, debugging};

/// Reference: https://epubs.siam.org/doi/10.1137/0202012
// TODO: describe general idea

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    Tree,
    Back,
    Killed,
}

#[derive(Debug, Clone)]
pub enum ComponentType {
    P, // bond
    S, // triangle
    R, // triconnected
}

#[derive(Debug, Clone)]
pub struct Component {
    edges: Vec<usize>,
    component_type: Option<ComponentType>,
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
                ComponentType::P
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

        eid
    }
    fn kill_edge(&mut self, eid: usize) {
        if cfg!(debug_assertions) {
            println!(
                "Killing edge {}: ({}, {})",
                eid, self.edges[eid].0, self.edges[eid].1
            );
        }
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

        self.deg[s] += 1;
        self.deg[t] += 1;

        self.par_edge[t] = Some(eid);
        self.par[t] = Some(s);
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
            return eid;
        }
        return usize::MAX;
    }
}

fn find_split(
    root: usize,
    u: usize,
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
    }

    fn ensure_highpoint(
        u: usize,
        tstack: &mut Vec<(usize, usize, usize)>,
        graph: &mut GraphInternal,
    ) {
        fn get_high(u: usize, graph: &mut GraphInternal) -> usize {
            while let Some(&eid) = graph.high[u].last() {
                if graph.edge_type[eid] == Some(EdgeType::Killed) {
                    graph.high[u].pop();

                    if cfg!(debug_assertions) {
                        println!("Removing killed edge {} from highpoint of {}", eid, u);
                    }
                } else {
                    return eid;
                }
            }
            0
        }

        while let Some(&(h, a, b)) = tstack.last() {
            if a != u && b != u && get_high(u, graph) > h {
                if cfg!(debug_assertions) {
                    println!(
                        "Popping tstack due to ensure_highpoints: ({}, {}, {})",
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
        loop {
            let (h, a, b) = if let Some(&last) = tstack.last() {
                last
            } else {
                (0, usize::MAX, 0)
            };

            let cond_1 = graph.num[u] != root && a == graph.num[u];
            let cond_2 =
                graph.deg[to] == 2 && graph.num[graph.first_alive(root, to)] > graph.num[to];

            if !(cond_1 || cond_2) {
                break;
            }
            if a == graph.num[u] && graph.par[graph.numrev[b]] == Some(u) {
                if cfg!(debug_assertions) {
                    println!("Popping {} {} from tstack: no inner vertex exists", a, b);
                }

                tstack.pop();
                continue;
            }

            let mut eab = usize::MAX;
            let mut evirt = usize::MAX;
            if cond_2 {
                to = graph.first_alive(root, to);

                if cfg!(debug_assertions) {
                    println!("Type 2 pair found (easy one) ({}, {})", a, to);
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
                    if graph.edges[e].0 == u && graph.edges[e].1 == to {
                        // a multiedge, it can't happen that .0 == to and .1 == u since that'd make a type-1 pair at 'to'
                        eab = estack.pop().unwrap();
                        graph.kill_edge(eab);
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
                        graph.kill_edge(e);

                        if [
                            graph.num[x].min(graph.num[y]),
                            graph.num[x].max(graph.num[y]),
                        ] == [graph.num[u], graph.num[to]]
                        {
                            eab = e;
                        } else {
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
                component.push_edge(evirt);

                evirt = graph.new_edge(u, to, None);
                component.push_edge(evirt);

                graph.kill_edge(eab);
            }

            estack.push(evirt);
            graph.make_tedge(evirt);
        }
    }
    fn check_type_1(
        root: usize,
        u: usize,
        to: usize,
        tstack: &mut Vec<(usize, usize, usize)>,
        estack: &mut Vec<usize>,
        graph: &mut GraphInternal,
        split_components: &mut Vec<Component>,
    ) {
    }

    let mut i = 0;
    while i < graph.adj[u].len() {
        let eid = graph.adj[u][i];
        if graph.edge_type[eid] == Some(EdgeType::Killed) {
            i += 1;
            continue; // skip killed edges
        }

        let to = graph.get_other(eid, u);
        if graph.starts_path[eid] {
            update_tstack(u, to, eid, tstack, graph);
        }

        if graph.edge_type[eid] == Some(EdgeType::Tree) {
            let mut empty_tstack = vec![];
            find_split(
                root,
                to,
                graph,
                estack,
                if graph.starts_path[eid] {
                    &mut empty_tstack
                } else {
                    tstack
                },
                split_components,
            );

            let push_eid = graph.par_edge[to].unwrap(); // eid could be killed by the multiple edge case in check_type_x
            estack.push(push_eid);

            check_type_2(root, u, to, tstack, estack, graph, split_components);
            check_type_1(root, u, to, tstack, estack, graph, split_components);

            ensure_highpoint(u, tstack, graph);
        } else {
        }

        i += 1;
    }
}

fn pathfinder_dfs(
    root: usize,
    u: usize,
    graph: &mut GraphInternal,
    newnum: &mut Vec<usize>,
    time: &mut usize,
) {
    let first_eid = graph.first_alive(root, u);

    let neighbors = graph.adj[u].clone(); // borrow checker doesn't like mutable borrow below

    for &eid in neighbors.iter() {
        let to = graph.get_other(eid, u);
        // no killed edges here

        if eid != first_eid {
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

fn dfs_1(u: usize, time: &mut usize, graph: &mut GraphInternal) {
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

        graph.deg[to] += 1;

        if graph.edge_type[eid].is_some() {
            continue; // already visited 
        }

        if graph.num[to] == usize::MAX {
            // tree edge
            graph.par_edge[to] = Some(eid);
            graph.par[to] = Some(u);
            graph.edge_type[eid] = Some(EdgeType::Tree);

            dfs_1(to, time, graph);

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

fn sort_pairs_upperbounded(edges: &mut Vec<(usize, usize)>, upper_bound: usize) {
    // TODO: implement a bucketsort here
    edges.sort();
}

fn handle_duplicate_edges(graph: &mut GraphInternal, split_components: &mut Vec<Component>) {
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
            graph.adj[t].push(eid); // add t->s edge as well, since we are not rooted yet

            component.push_edge(i);
            graph.edge_type[i] = Some(EdgeType::Killed);

            while i + 1 < len && graph.edges[i + 1] == graph.edges[i] {
                i += 1;

                component.push_edge(i);
                graph.edge_type[i] = Some(EdgeType::Killed);
            }

            split_components.push(component);
        } else {
            graph.adj[s].push(i);
            graph.adj[t].push(i); // add both directions, since we are not rooted yet
        }

        i += 1;
    }
}

pub fn get_triconnected_components(in_graph: &UnGraph) -> Vec<Component> {
    let n = in_graph.node_count();
    let m = in_graph.edge_count();
    let root = 0;

    let mut split_components = Vec::new();

    if cfg!(debug_assertions) {
        println!("{} nodes, {} edges", n, m);
    }

    // TODO: input graph should be biconnected, assert it here?

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

        sort_pairs_upperbounded(&mut graph.edges, n);
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
        dfs_1(root, &mut time, &mut graph);

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
                3 * to + 1
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
            &mut graph,
            &mut estack,
            &mut tstack,
            &mut split_components,
        );

        if !estack.is_empty() {
            let mut component = Component::new(None);
            while let Some(eid) = estack.pop() {
                component.push_edge(eid);
                graph.edge_type[eid] = Some(EdgeType::Killed);
            }
            component.commit(&mut split_components);
        }
    }

    // merge S and P nodes
    {
        // TODO
    }

    split_components
}
