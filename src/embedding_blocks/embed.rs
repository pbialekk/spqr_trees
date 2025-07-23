use petgraph::visit::NodeIndexable;

use crate::{
    EdgeLabel,
    embedding_blocks::{
        acceptable_adj::make_adjacency_lists_acceptable,
        structures::{GraphInternal, LrOrientation},
    },
    types::DiGraph,
};

/// Implements the DFS3 algorithm from the reference.
pub fn embed_graph(
    graph: &mut GraphInternal,
    lr_stuff: &mut LrOrientation,
    roots: &Vec<usize>,
) -> DiGraph {
    fn sign(eid: usize, lr_stuff: &mut LrOrientation) -> i8 {
        if lr_stuff.ref_edge[eid] != usize::MAX {
            lr_stuff.side[eid] *= sign(lr_stuff.ref_edge[eid], lr_stuff);
            lr_stuff.ref_edge[eid] = usize::MAX;
        }
        return lr_stuff.side[eid];
    }

    for i in 0..graph.m {
        graph.nesting_depth[i] *= sign(i, lr_stuff) as isize;
    }

    make_adjacency_lists_acceptable(graph);

    let mut left = vec![Vec::new(); graph.n];
    let mut right = vec![Vec::new(); graph.n];

    let mut embedded_g = DiGraph::new();
    for u in 0..graph.n {
        embedded_g.add_node(u.try_into().unwrap());
    }

    for &u in roots {
        dfs3(graph, lr_stuff, &mut left, &mut right, u, &mut embedded_g);
    }

    embedded_g
}

pub fn dfs3(
    graph: &mut GraphInternal,
    lr_stuff: &LrOrientation,
    left: &mut [Vec<usize>],
    right: &mut [Vec<usize>],
    u: usize,
    embedded_g: &mut DiGraph,
) {
    fn add_edge(graph: &mut GraphInternal, embedded_g: &mut DiGraph, u: usize, v: usize) {
        if let Some(count) = graph.edge_counts.get_mut(&(u, v)) {
            for _ in 0..*count {
                embedded_g.add_edge(
                    embedded_g.from_index(u),
                    embedded_g.from_index(v),
                    EdgeLabel::Real,
                );
            }
            *count = 0;
        }
    }

    let neis = graph.adj[u].clone();

    for &eid in neis.iter() {
        let to = graph.get_other_vertex(eid, u);

        if Some(eid) == graph.parent[to] {
            add_edge(graph, embedded_g, to, u);
            dfs3(graph, lr_stuff, left, right, to, embedded_g);

            // stond przyszli

            for eid in left[u].iter().rev() {
                add_edge(graph, embedded_g, u, *eid);
            }
            add_edge(graph, embedded_g, u, to);
            for eid in right[u].iter().rev() {
                add_edge(graph, embedded_g, u, *eid);
            }

            left[u].clear();
            right[u].clear();
        } else {
            add_edge(graph, embedded_g, u, to);

            if lr_stuff.side[eid] == 1 {
                right[to].push(u);
            } else {
                left[to].push(u);
            }
        }
    }
    add_edge(graph, embedded_g, u, u);
}
