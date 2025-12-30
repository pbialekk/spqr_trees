use crate::types::DiGraph;
use petgraph::visit::NodeIndexable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Blue,
    Green,
    Black,
}

impl Color {
    pub fn index(&self) -> usize {
        match self {
            Color::Red => 0,
            Color::Blue => 1,
            Color::Green => 2,
            Color::Black => 3,
        }
    }
}

pub struct SchnyderTree {
    pub parent: Vec<usize>,
    pub children: Vec<Vec<usize>>,
    pub dep: Vec<usize>,
    pub subsz: Vec<usize>,
    pub pathdp: [Vec<usize>; 3],
    pub root: usize,
}

impl SchnyderTree {
    pub fn new(n: usize, root: usize) -> Self {
        Self {
            parent: vec![usize::MAX; n],
            children: vec![Vec::new(); n],
            dep: vec![0; n],
            subsz: vec![0; n],
            pathdp: [vec![0; n], vec![0; n], vec![0; n]],
            root,
        }
    }

    pub fn add_edge(&mut self, u: usize, v: usize) {
        // v is parent of u in this tree, i.e., edge u -> v
        self.children[v].push(u);
        self.parent[u] = v;
    }
}

pub struct DrawingResult {
    pub coordinates: Vec<(i64, i64)>,
    pub edge_colors: Vec<(usize, usize, Color)>,
}

pub fn draw(g: &DiGraph) -> DrawingResult {
    let n = g.node_count();

    let f0 = 0;

    // We need neighbors of f0.
    let neighbors: Vec<usize> = g
        .neighbors(g.from_index(f0))
        .map(|x| g.to_index(x))
        .collect();

    if neighbors.len() < 2 {
        panic!("Graph must be triangulated (degree >= 2)");
    }

    // Since it's triangulated, any two consecutive neighbors of a node form a face with that node.
    let f1 = neighbors[0];
    let f2 = neighbors[1];

    let f = [f0, f1, f2];

    let mut prev = vec![usize::MAX; n];
    let mut next = vec![usize::MAX; n];

    // Initialize outer boundary: f0 -> f2 -> f1 -> f0
    next[f[0]] = f[2];
    prev[f[2]] = f[0];
    next[f[2]] = f[1];
    prev[f[1]] = f[2];
    next[f[1]] = f[0];
    prev[f[0]] = f[1];

    let mut ch = vec![0; n];
    let mut out = vec![false; n];
    out[f[0]] = true;
    out[f[1]] = true;
    out[f[2]] = true;

    let mut used = vec![false; n];

    let mut cands = Vec::new();

    cands.push(f[2]);

    let mut trees = [
        SchnyderTree::new(n, f[0]),
        SchnyderTree::new(n, f[1]),
        SchnyderTree::new(n, f[2]),
    ];

    trees[0].add_edge(f[1], f[0]);
    trees[1].add_edge(f[0], f[1]);
    trees[2].add_edge(f[0], f[2]);
    trees[2].add_edge(f[1], f[2]);

    let mut edge_colors_list = Vec::new();
    // For drawing purposes, the outer face edges should be black.
    edge_colors_list.push((f[1], f[0], Color::Black));
    edge_colors_list.push((f[0], f[1], Color::Black));
    edge_colors_list.push((f[0], f[2], Color::Black));
    edge_colors_list.push((f[1], f[2], Color::Black));

    for _ in (2..n).rev() {
        let mut u = usize::MAX;
        while let Some(cand) = cands.pop() {
            if !used[cand] && out[cand] && ch[cand] == 0 && cand != f[0] && cand != f[1] {
                u = cand;
                break;
            }
        }

        if u == usize::MAX {
            break;
        }

        used[u] = true;
        out[u] = false;

        let mut ws = Vec::new();
        for neighbor in g.neighbors(g.from_index(u)) {
            let v = g.to_index(neighbor);
            if !used[v] {
                ws.push(v);
            }
        }

        if ws.is_empty() {
            continue;
        }

        let p = prev[u];
        let n_node = next[u];

        let p_pos = ws.iter().position(|&x| x == p);
        if p_pos.is_none() {
            panic!("Prev neighbor not found in unused neighbors list");
        }
        let p_idx = p_pos.unwrap();
        ws.rotate_left(p_idx);

        if ws.get(1) == Some(&n_node) {
            if ws.len() > 2 {
                ws[1..].reverse();
            }
        }
        for i in 0..ws.len() - 1 {
            let u_curr = ws[i];
            let v_curr = ws[i + 1];
            next[u_curr] = v_curr;
            prev[v_curr] = u_curr;
        }

        trees[0].add_edge(u, ws[0]);
        let color_red = if u == f[2] { Color::Black } else { Color::Red };
        edge_colors_list.push((u, ws[0], color_red));

        trees[1].add_edge(u, ws[ws.len() - 1]);
        let color_blue = if u == f[2] { Color::Black } else { Color::Blue };
        edge_colors_list.push((u, ws[ws.len() - 1], color_blue));

        for j in 1..ws.len() - 1 {
            let to = ws[j];
            trees[2].add_edge(to, u);
            edge_colors_list.push((to, u, Color::Green));

            out[to] = true;
            cands.push(to);

            for neighbor in g.neighbors(g.from_index(to)) {
                let tow = g.to_index(neighbor);
                if tow == ws[j - 1] || tow == ws[j + 1] {
                    continue;
                }
                if out[tow] {
                    ch[tow] += 1;
                    ch[to] += 1;
                }
            }
        }

        if ws.len() == 2 {
            if ch[ws[0]] > 0 {
                ch[ws[0]] -= 1;
            }
            if ch[ws[1]] > 0 {
                ch[ws[1]] -= 1;
            }
            cands.push(ws[0]);
            cands.push(ws[1]);
        }
    }

    fn dfs(t_idx: usize, u: usize, trees: &mut [SchnyderTree; 3]) {
        trees[t_idx].subsz[u] = 1;
        let children = trees[t_idx].children[u].clone();
        for &to in &children {
            trees[t_idx].dep[to] = trees[t_idx].dep[u] + 1;
            dfs(t_idx, to, trees);
            trees[t_idx].subsz[u] += trees[t_idx].subsz[to];
        }
    }

    for i in 0..3 {
        dfs(i, trees[i].root, &mut trees);
    }

    fn compute_pathdp(t_idx: usize, u: usize, p: usize, trees: &mut [SchnyderTree; 3]) {
        for j in 0..3 {
            if j == t_idx {
                continue;
            }
            let val_p = if u == p { 0 } else { trees[t_idx].pathdp[j][p] };
            let val_subsz = trees[j].subsz[u];
            trees[t_idx].pathdp[j][u] = val_p + val_subsz;
        }

        let children = trees[t_idx].children[u].clone();
        for &to in &children {
            compute_pathdp(t_idx, to, u, trees);
        }
    }

    for i in 0..3 {
        compute_pathdp(i, trees[i].root, trees[i].root, &mut trees);
    }

    let mut coords = Vec::new();
    for u in 0..n {
        let mut c = [0i64; 3];
        for i in 0..3 {
            let ip1 = (i + 1) % 3;
            let im1 = (i + 2) % 3;

            let p1 = trees[ip1].pathdp[i][u] as i64;
            let p2 = trees[im1].pathdp[i][u] as i64;
            let s = trees[i].subsz[u] as i64;

            let mut region = p1 + p2 - s;
            if u == f[i] {
                region -= 2;
            }

            let im1_dep = trees[im1].dep[u] as i64;
            c[i] = region - (im1_dep + 1);
        }
        coords.push((c[1], c[2]));
    }

    DrawingResult {
        coordinates: coords,
        edge_colors: edge_colors_list,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drawing_blocks::triangulate::triangulate;
    use crate::embedding::is_planar;
    use crate::testing::graph_enumerator::GraphEnumeratorState;
    use petgraph::visit::EdgeRef;

    // Helper functions for geometry
    fn ccw(a: (i64, i64), b: (i64, i64), c: (i64, i64)) -> i64 {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    }

    fn on_segment(a: (i64, i64), b: (i64, i64), c: (i64, i64)) -> bool {
        // Check bounding box
        c.0 >= a.0.min(b.0) && c.0 <= a.0.max(b.0) && c.1 >= a.1.min(b.1) && c.1 <= a.1.max(b.1)
    }

    fn do_lines_intersect(p1: (i64, i64), p2: (i64, i64), p3: (i64, i64), p4: (i64, i64)) -> bool {
        let o1 = ccw(p1, p2, p3);
        let o2 = ccw(p1, p2, p4);
        let o3 = ccw(p3, p4, p1);
        let o4 = ccw(p3, p4, p2);

        // General crossing
        if o1 * o2 < 0 && o3 * o4 < 0 {
            return true;
        }

        // Collinear cases
        if o1 == 0 && on_segment(p1, p2, p3) {
            return true;
        }
        if o2 == 0 && on_segment(p1, p2, p4) {
            return true;
        }
        if o3 == 0 && on_segment(p3, p4, p1) {
            return true;
        }
        if o4 == 0 && on_segment(p3, p4, p2) {
            return true;
        }

        false
    }

    // Helper to check for overlapping edges that shouldn't differ only by endpoint
    fn check_intersections(g: &DiGraph, drawing: &DrawingResult) {
        let edges: Vec<_> = g
            .edge_references()
            .map(|e| (g.to_index(e.source()), g.to_index(e.target())))
            .collect();
        let n = g.node_count();

        // 1. Check interactions between disjoint edges
        for i in 0..edges.len() {
            for j in i + 1..edges.len() {
                let (u1, v1) = edges[i];
                let (u2, v2) = edges[j];

                // Ignore edges sharing endpoints
                if u1 == u2 || u1 == v2 || v1 == u2 || v1 == v2 {
                    continue;
                }

                let p1 = drawing.coordinates[u1];
                let p2 = drawing.coordinates[v1];
                let p3 = drawing.coordinates[u2];
                let p4 = drawing.coordinates[v2];

                if do_lines_intersect(p1, p2, p3, p4) {
                    panic!(
                        "Disjoint edges interact! {:?} {:?} at coords {:?} {:?} {:?} {:?}",
                        edges[i], edges[j], p1, p2, p3, p4
                    );
                }
            }
        }

        // 2. Check vertex lying on edge
        for i in 0..n {
            let pv = drawing.coordinates[i];
            for &(u, v) in &edges {
                if i == u || i == v {
                    continue;
                }
                let pu = drawing.coordinates[u];
                let pv_end = drawing.coordinates[v];

                if ccw(pu, pv_end, pv) == 0 && on_segment(pu, pv_end, pv) {
                    panic!(
                        "Vertex {} lies on edge {:?}! Coords: {:?} on {:?}-{:?}",
                        i,
                        (u, v),
                        pv,
                        pu,
                        pv_end
                    );
                }
            }
        }
    }

    #[test]
    fn test_schnyder_small_graphs() {
        // Enumerate small graphs, triangulate, draw, verify.
        for n in 3..=6 {
            let mut enumerator = GraphEnumeratorState {
                n,
                mask: 0,
                last_mask: 1 << (n * (n - 1) / 2),
            };

            while let Some(g) = enumerator.next() {
                let n = g.node_count();
                let (planar, _) = is_planar(&g, false);
                if planar {
                    let triangulated = triangulate(&g);
                    let drawing = draw(&triangulated);

                    // Verify coordinates are non-negative
                    for (x, y) in &drawing.coordinates {
                        assert!(*x >= 0 && *x <= (n as i64) - 2);
                        assert!(*y >= 0 && *y <= (n as i64) - 2);
                    }

                    // Verify edge intersections
                    check_intersections(&triangulated, &drawing);
                }
            }
        }
    }
}
