use hashbrown::HashMap;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Interval {
    pub ends: Option<(usize, usize)>,
}
impl Interval {
    pub fn new(lo: usize, hi: usize) -> Self {
        Interval {
            ends: Some((lo, hi)),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.ends.is_none()
    }
    pub fn empty() -> Self {
        Interval { ends: None }
    }
    pub fn lo(&self) -> usize {
        self.ends.unwrap().0
    }
    pub fn hi(&self) -> usize {
        self.ends.unwrap().1
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConflictPair {
    pub l: Interval,
    pub r: Interval,
}
impl ConflictPair {
    pub fn empty() -> Self {
        ConflictPair {
            l: Interval::empty(),
            r: Interval::empty(),
        }
    }
    pub fn flip(&mut self) {
        std::mem::swap(&mut self.l, &mut self.r);
    }
    pub fn is_empty(&self) -> bool {
        self.l.is_empty() && self.r.is_empty()
    }
    fn lowest(&self, g: &GraphInternal) -> usize {
        match (self.l.is_empty(), self.r.is_empty()) {
            (true, false) => g.low1[self.r.lo()],
            (false, true) => g.low1[self.l.lo()],
            (false, false) => {
                let l_low = g.low1[self.l.lo()];
                let r_low = g.low1[self.r.lo()];
                l_low.min(r_low)
            }
            (true, true) => panic!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphInternal {
    pub n: usize,                   // number of vertices
    pub m: usize,                   // number of edges
    pub adj: Vec<Vec<usize>>,       // adjacency list, edges are stored as indices in `edges`
    pub edges: Vec<(usize, usize)>, // edges in the form (source, target)

    pub low1: Vec<usize>,
    pub low2: Vec<usize>,
    pub nesting_depth: Vec<isize>,

    pub parent: Vec<Option<usize>>, // parent edge of each vertex in the DFS tree
    pub height: Vec<usize>,         // height of the vertex in the DFS tree

    pub edge_counts: HashMap<(usize, usize), usize>, // count of edges between pairs of vertices
}

impl GraphInternal {
    pub fn new(n: usize, m: usize) -> Self {
        GraphInternal {
            n,
            m,
            adj: vec![Vec::new(); n],
            edges: Vec::with_capacity(m),
            low1: vec![usize::MAX; m],
            low2: vec![usize::MAX; m],
            nesting_depth: vec![isize::MAX; m],
            parent: vec![None; n],
            height: vec![usize::MAX; n],
            edge_counts: HashMap::new(),
        }
    }

    pub fn get_other_vertex(&self, eid: usize, u: usize) -> usize {
        let (s, t) = self.edges[eid];
        if s == u { t } else { s }
    }

    pub fn add_edge(&mut self, u: usize, v: usize) {
        let count = self.edge_counts.entry((u, v)).or_insert(0);
        if *count == 0 && u != v {
            let eid = self.edges.len();
            self.adj[u].push(eid);
            self.adj[v].push(eid);
            self.edges.push((u, v));
        } else {
            self.m -= 1;
        }
        *count += 1;

        if u != v {
            let count_other = self.edge_counts.entry((v, u)).or_insert(0);
            *count_other += 1;
        }
    }
}

pub struct LrOrientation {
    pub stack_bottom: Vec<Option<ConflictPair>>,
    pub lowpt_edge: Vec<usize>,
    pub ref_edge: Vec<usize>,
    pub stack: Vec<ConflictPair>,
    pub side: Vec<i8>,
}
impl LrOrientation {
    pub fn new(_: usize, m: usize) -> Self {
        LrOrientation {
            stack_bottom: vec![None; m],
            lowpt_edge: vec![0; m],
            ref_edge: vec![usize::MAX; m],
            stack: Vec::new(),
            side: vec![1; m], // +1 for right
        }
    }

    pub fn merge_intervals(&mut self, p: &mut Interval, q: &mut Interval) {
        if let Some((p_lo, _)) = p.ends.as_mut() {
            if let Some((lo, hi)) = q.ends {
                self.ref_edge[*p_lo] = hi;
                *p_lo = lo;
            }
        } else {
            p.ends = q.ends;
        }
    }

    /// Returns true if the merge was successful, false if it failed due to conflicting intervals.
    pub fn merge(&mut self, g: &GraphInternal, eid: usize) -> bool {
        let u = g.edges[eid].0;
        let par_eid = g.parent[u].unwrap();

        let mut p = ConflictPair::empty();
        loop {
            // All of intervals from the eid subtree need to be embedded on the same side because of the fundamental cycle u -> e1 -> ...
            let mut q = self.stack.pop().unwrap();
            if !q.l.is_empty() {
                q.flip();
            }
            if !q.l.is_empty() {
                return false;
            }

            let lo = q.r.lo();

            if g.low1[lo] > g.low1[par_eid] {
                self.merge_intervals(&mut p.r, &mut q.r);
            } else {
                self.ref_edge[lo] = self.lowpt_edge[par_eid];
            }

            if self.stack.last().cloned() == self.stack_bottom[eid] {
                break;
            }
        }

        // and now merge p with previous constraints
        while let Some(q) = self.stack.last() {
            fn conflicting(interval: Interval, b: usize, g: &GraphInternal) -> bool {
                !interval.is_empty() && g.low1[interval.hi()] > g.low1[b]
            }

            if !(conflicting(q.l, eid, g) || conflicting(q.r, eid, g)) {
                break;
            }

            let mut q = self.stack.pop().unwrap();

            if conflicting(q.r, eid, g) {
                q.flip();
            }
            if conflicting(q.r, eid, g) {
                return false;
            }

            self.merge_intervals(&mut p.r, &mut q.r);
            self.merge_intervals(&mut p.l, &mut q.l);
        }

        if !p.is_empty() {
            self.stack.push(p);
        }

        true
    }

    fn trim_interval(&mut self, p: &mut Interval, u: usize, g: &GraphInternal, other_p: &Interval) {
        if p.is_empty() {
            return;
        }

        while p.hi() != usize::MAX && g.edges[p.hi()].1 == u {
            p.ends = Some((p.lo(), self.ref_edge[p.hi()]));
        }

        if p.hi() == usize::MAX {
            if !other_p.is_empty() {
                self.ref_edge[p.lo()] = other_p.lo();
            }
            self.side[p.lo()] = -1;
            *p = Interval::empty();
        }
    }

    /// Erases conflicting intervals that are not needed anymore.
    pub fn trim(&mut self, g: &GraphInternal, par_eid: usize) {
        let u = g.edges[par_eid].0;

        while let Some(q) = self.stack.last() {
            if q.lowest(g) != g.height[u] {
                break;
            }
            let q = self.stack.pop().unwrap();

            if !q.l.is_empty() {
                self.side[q.l.lo()] = -1;
            }
        }

        if !self.stack.is_empty() {
            let mut p = self.stack.pop().unwrap();
            self.trim_interval(&mut p.l, u, g, &p.r);
            self.trim_interval(&mut p.r, u, g, &p.l);

            if !p.is_empty() {
                self.stack.push(p);
            }
        }
    }
}
