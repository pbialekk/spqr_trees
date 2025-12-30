use super::faces::get_faces;
use crate::{
    UnGraph,
    embedding::is_planar,
    types::{DiGraph, EdgeLabel},
};
use petgraph::visit::{EdgeRef, NodeIndexable};

fn to_ungraph(graph: &DiGraph) -> UnGraph {
    let mut g = UnGraph::new_undirected();
    for _ in 0..graph.node_count() {
        g.add_node(0);
    }
    for e in graph.edge_references() {
        let u = e.source();
        let v = e.target();
        if u.index() < v.index() {
            g.add_edge(u, v, EdgeLabel::Real);
        }
    }
    g
}

fn do_embed(graph: &mut DiGraph) {
    let g_un = to_ungraph(graph);
    let (is_planar, embedding) = is_planar(&g_un, false);
    *graph = embedding;
}

pub fn triangulate(graph: &UnGraph) -> DiGraph {
    let (is_planar, mut g) = is_planar(graph, false);
    connect_components(&mut g);
    do_embed(&mut g);
    make_biconnected(&mut g);
    do_embed(&mut g);
    triangulate_faces(&mut g);
    do_embed(&mut g);
    g
}

fn connect_components(g: &mut DiGraph) {
    let n = g.node_count();
    let mut visited = vec![false; n];
    let mut roots = Vec::new();

    for i in 0..n {
        if !visited[i] {
            roots.push(i);
            let mut stack = vec![i];
            visited[i] = true;
            while let Some(u) = stack.pop() {
                for neighbor in g.neighbors(g.from_index(u)) {
                    let v = g.to_index(neighbor);
                    if !visited[v] {
                        visited[v] = true;
                        stack.push(v);
                    }
                }
            }
        }
    }

    for i in 0..roots.len().saturating_sub(1) {
        let u = roots[i];
        let v = roots[i + 1];
        g.add_edge(g.from_index(u), g.from_index(v), EdgeLabel::Real);
        g.add_edge(g.from_index(v), g.from_index(u), EdgeLabel::Real);
    }
}

fn make_biconnected(g: &mut DiGraph) {
    let faces = get_faces(g);
    let n = g.node_count();

    let mut visited = vec![false; n];
    let mut visited_nodes = Vec::new();

    for face in faces {
        if face.order.len() < 3 {
            continue;
        }

        for &v in &visited_nodes {
            visited[v] = false;
        }
        visited_nodes.clear();

        let mut f_vec: Vec<usize> = face.order.clone();
        if f_vec.len() < 3 {
            continue;
        }

        let first = f_vec[0];
        let second = f_vec[1];
        let mut idx = 0;

        loop {
            if f_vec.len() < 3 {
                break;
            }

            let a_idx = idx % f_vec.len();
            let b_idx = (idx + 1) % f_vec.len();
            let c_idx = (idx + 2) % f_vec.len();

            let a = f_vec[a_idx];
            let b = f_vec[b_idx];
            let c = f_vec[c_idx];

            if b == first && c == second {
                break;
            }

            if visited[b] {
                g.add_edge(g.from_index(a), g.from_index(c), EdgeLabel::Real);
                g.add_edge(g.from_index(c), g.from_index(a), EdgeLabel::Real);
                f_vec.remove(b_idx);
            } else {
                visited[b] = true;
                visited_nodes.push(b);
                idx += 1;
            }
        }
    }
}

fn triangulate_faces(g: &mut DiGraph) {
    let faces = get_faces(g);
    let n = g.node_count();

    let mut visited = vec![false; n];
    let mut visited_nodes = Vec::new();

    for face in faces {
        let mut f_vec: Vec<usize> = face.order.iter().cloned().collect();
        if f_vec.len() < 3 {
            continue;
        }

        let mut start_idx = 0;
        let mut min_deg = usize::MAX;
        let mut min_v = usize::MAX;

        for (i, &v) in f_vec.iter().enumerate() {
            let deg = g.edges(g.from_index(v)).count();
            if deg < min_deg || (deg == min_deg && v < min_v) {
                min_deg = deg;
                min_v = v;
                start_idx = i;
            }
        }

        f_vec.rotate_left(start_idx);

        for &v in &visited_nodes {
            visited[v] = false;
        }
        visited_nodes.clear();

        let a_node = f_vec[0];
        for neighbor in g.neighbors(g.from_index(a_node)) {
            let neighbor_idx = g.to_index(neighbor);
            visited[neighbor_idx] = true;
            visited_nodes.push(neighbor_idx);
        }

        loop {
            if f_vec.len() < 4 {
                break;
            }

            let a = f_vec[0];
            let b = f_vec[1];
            let c = f_vec[2];
            let d = f_vec[3];

            if visited[c] {
                g.add_edge(g.from_index(b), g.from_index(d), EdgeLabel::Real);
                g.add_edge(g.from_index(d), g.from_index(b), EdgeLabel::Real);
                f_vec.remove(2);
            } else {
                g.add_edge(g.from_index(a), g.from_index(c), EdgeLabel::Real);
                g.add_edge(g.from_index(c), g.from_index(a), EdgeLabel::Real);
                visited[c] = true;
                visited_nodes.push(c);
                f_vec.remove(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::is_planar;
    use crate::testing::graph_enumerator::GraphEnumeratorState;
    use petgraph::visit::EdgeRef;

    fn is_simple(g: &UnGraph) -> bool {
        let n = g.node_count();
        // Check self-loops
        for e in g.edge_references() {
            if e.source() == e.target() {
                return false;
            }
        }
        let mut adj = vec![std::collections::HashSet::new(); n];
        for e in g.edge_references() {
            let u = g.to_index(e.source());
            let v = g.to_index(e.target());
            let (min, max) = if u < v { (u, v) } else { (v, u) };
            if !adj[min].insert(max) {
                return false;
            }
        }
        true
    }

    #[test]
    fn test_triangulation_exhaustive() {
        for n in 3..=6 {
            let mut enumerator = GraphEnumeratorState {
                n,
                mask: 0,
                last_mask: 1 << (n * (n - 1) / 2),
            };

            while let Some(g) = enumerator.next() {
                // We only care if input is planar.
                // Disconnected planar graphs are valid inputs for triangulate as we implemented component connection.
                let (planar, _) = is_planar(&g, false);
                if planar {
                    let tri_g_directed = triangulate(&g);
                    let mut tri_g = UnGraph::new_undirected();
                    for _ in 0..tri_g_directed.node_count() {
                        tri_g.add_node(0);
                    }
                    for e in tri_g_directed.edge_references() {
                        let u = e.source();
                        let v = e.target();
                        if u.index() < v.index() {
                            tri_g.add_edge(u, v, EdgeLabel::Real);
                        }
                    }

                    // Verify planarity
                    let (is_p, _) = is_planar(&tri_g, false);
                    assert!(is_p, "Triangulated graph must be planar. Original n={}", n);

                    // Verify edge count: 3n - 6 for n >= 3
                    let m = tri_g.edge_count();
                    dbg!(&tri_g);
                    assert_eq!(
                        m,
                        3 * n - 6,
                        "Triangulated graph must have 3n-6 edges. n={}, m={}",
                        n,
                        m
                    );

                    // Verify simplicity
                    assert!(
                        is_simple(&tri_g),
                        "Triangulated graph must be simple. n={}",
                        n
                    );
                }
            }
        }
    }
}
