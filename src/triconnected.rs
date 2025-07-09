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

    pub fn commit(&mut self) {
        if self.component_type.is_none() {
            self.component_type = Some(if self.edges.len() >= 4 {
                ComponentType::R
            } else {
                ComponentType::P
            });
        }
    }
}

// holds num[v], par[v], low1[v], low2[v], sub[v], deg[v]
#[derive(Debug, Clone)]
struct GraphInternal {
    adj: Vec<Vec<usize>>, // adjacency list, edges are stored as indices in `edges`
    edges: Vec<(usize, usize)>, // edges in the form (source, target)
    edge_type: Vec<Option<EdgeType>>, // edge type, None means not visited yet

    num: Vec<usize>,

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
    tstack: &mut Vec<usize>,
    split_components: &mut Vec<Component>,
) {
    // TODO
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

        let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); 3 * (n) + 2];

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
            component.commit();
            split_components.push(component);
        }
    }

    // merge S and P nodes
    {
        // TODO
    }

    split_components
}
