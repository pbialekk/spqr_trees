use petgraph::visit::EdgeRef;
use petgraph::visit::NodeIndexable;
use std::collections::HashMap;

use crate::types::DiGraph;

/// Represents a face in the dual graph of a planar graph.
#[derive(Debug, Clone)]
pub struct Face {
    /// Order of vertices (clockwise or counterclockwise for outer face)
    pub order: Vec<usize>,
}

/// Assumes that graph is properly embedded
pub fn get_faces(graph: &DiGraph) -> Vec<Face> {
    let n = graph.node_count();

    let mut edge_map = HashMap::new();
    for e in graph.edge_references() {
        let u = graph.to_index(e.source());
        let v = graph.to_index(e.target());
        edge_map.insert((u, v), e.id());
    }

    let mut adj = vec![Vec::new(); n];
    for u in 0..n {
        let u_idx = graph.from_index(u);
        for e in graph.edges(u_idx) {
            adj[u].push(e.id());
        }
    }

    let mut used = HashMap::new();
    let mut faces = Vec::new();

    for u in 0..n {
        for &eid in &adj[u] {
            if used.contains_key(&eid) {
                continue;
            }

            let mut face_nodes = Vec::new();
            let mut curr_eid = eid;

            loop {
                used.insert(curr_eid, true);
                let (src, dst) = graph.edge_endpoints(curr_eid).unwrap();
                let u_idx = graph.to_index(src);
                let v_idx = graph.to_index(dst);

                face_nodes.push(u_idx);

                let twin_eid = edge_map.get(&(v_idx, u_idx)).expect("Twin edge not found");
                let v_adj = &adj[v_idx];
                let idx_in_adj = v_adj
                    .iter()
                    .position(|&x| x == *twin_eid)
                    .expect("Edge not in adj");

                let next_idx = (idx_in_adj + 1) % v_adj.len();
                let next_eid = v_adj[next_idx];

                curr_eid = next_eid;

                if curr_eid == eid {
                    break;
                }
            }
            faces.push(Face { order: face_nodes });
        }
    }

    faces
}
