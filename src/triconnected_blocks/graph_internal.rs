use std::mem::swap;

use petgraph::visit::EdgeRef;

use crate::{UnGraph, triconnected_blocks::outside_structures::EdgeType};

#[derive(Debug, Clone)]
pub(crate) struct GraphInternal {
    pub n: usize,                         // number of vertices
    pub m: usize,                         // number of edges
    pub adj: Vec<Vec<usize>>,             // adjacency list, edges are stored as indices in `edges`
    pub edges: Vec<(usize, usize)>,       // edges in the form (source, target)
    pub edge_type: Vec<Option<EdgeType>>, // edge type, None means not visited yet

    pub num: Vec<usize>,

    pub par_edge: Vec<Option<usize>>, // edge id of the parent edge in DFS tree
    pub par: Vec<Option<usize>>,
    pub low1: Vec<usize>,
    pub low2: Vec<usize>,
    pub sub: Vec<usize>,
    pub deg: Vec<usize>,

    pub high: Vec<Vec<usize>>, // pathfinder starts here
    pub numrev: Vec<usize>,    // reverse mapping from num to original vertex
    pub starts_path: Vec<bool>,
}
impl GraphInternal {
    pub fn from_petgraph(graph: &UnGraph) -> Self {
        let n = graph.node_count();
        let mut ret = Self::new(n, 0);

        for e in graph.edge_references() {
            let (mut s, mut t) = (e.source().index(), e.target().index());
            if s > t {
                swap(&mut s, &mut t);
            }
            ret.new_edge(s, t, None);
        }

        ret
    }
    pub fn new(n: usize, m: usize) -> Self {
        Self {
            n,
            m,
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
            starts_path: Vec::new(),
        }
    }
    pub fn new_edge(&mut self, s: usize, t: usize, put_type: Option<EdgeType>) -> usize {
        let eid = self.edges.len();

        self.edges.push((s, t));
        self.edge_type.push(put_type);
        self.adj[s].push(eid);
        self.starts_path.push(false);
        self.deg[s] += 1;
        self.deg[t] += 1;
        self.m += 1;

        eid
    }
    pub fn remove_edge(&mut self, eid: usize) {
        debug_assert!(self.edge_type[eid] != Some(EdgeType::Killed));

        self.edge_type[eid] = Some(EdgeType::Killed);
        let (s, t) = self.edges[eid];
        self.deg[s] -= 1;
        self.deg[t] -= 1;
    }
    pub fn make_tedge(&mut self, eid: usize) {
        debug_assert!(self.edge_type[eid] == None);

        self.edge_type[eid] = Some(EdgeType::Tree);
        let (s, t) = self.edges[eid];

        self.par_edge[t] = Some(eid);
        self.par[t] = Some(s);
    }
    pub fn make_bedge(&mut self, eid: usize) {
        debug_assert!(self.edge_type[eid] == None);

        self.edge_type[eid] = Some(EdgeType::Back);
        let (s, t) = self.edges[eid];

        if self.get_high(s) < self.num[s] {
            self.high[t].push(eid);
        }
    }
    pub fn get_other_vertex(&self, eid: usize, u: usize) -> usize {
        let (s, t) = self.edges[eid];
        if s == u { t } else { s }
    }
    pub fn first_alive(&self, root: usize, u: usize) -> Option<usize> {
        if u == root {
            return None;
        }
        for &eid in &self.adj[u] {
            if self.edge_type[eid] == Some(EdgeType::Killed) {
                continue;
            }
            return Some(self.edges[eid].1);
        }
        None
    }
    pub fn get_high(&mut self, u: usize) -> usize {
        while let Some(&eid) = self.high[u].last() {
            if self.edge_type[eid] == Some(EdgeType::Killed) {
                self.high[u].pop();
            } else {
                return self.num[self.get_other_vertex(eid, u)];
            }
        }
        0
    }
}
