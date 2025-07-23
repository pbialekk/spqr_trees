use std::collections::BTreeSet;

use crate::{EdgeLabel, UnGraph, embedding::is_planar, types::DiGraph};
use petgraph::algo::is_isomorphic;

/// Given a non-planar graph, extract it's corresponding kuratowski subgraph. Works in O(n^2)
pub fn get_counterexample(mut graph: UnGraph, with_counterexample: bool) -> DiGraph {
    let mut ret = DiGraph::new();
    let mut ret_undir = UnGraph::new_undirected();

    for v in graph.node_indices() {
        ret.add_node(v.index().try_into().unwrap());
        ret_undir.add_node(v.index().try_into().unwrap());
    }
    if !with_counterexample {
        return ret;
    }

    while graph.edge_count() > 0 {
        let eid = graph.edge_indices().next().unwrap();
        let (u, v) = graph.edge_endpoints(eid).unwrap();

        graph.remove_edge(eid);

        let mut graph_test = graph.clone();
        for ret_edge in ret.edge_indices() {
            let (ret_u, ret_v) = ret.edge_endpoints(ret_edge).unwrap();
            graph_test.add_edge(ret_u, ret_v, EdgeLabel::Real);
        }

        if is_planar(&graph_test, false).0 {
            ret.add_edge(u, v, EdgeLabel::Real);
            ret.add_edge(v, u, EdgeLabel::Real);
            ret_undir.add_edge(u, v, EdgeLabel::Real);
        }
    }

    assert!(!is_planar(&ret_undir, false).0);

    // it's obvious that we only take edges that are part of the input graph.

    // we have to assert that ret_undir is a homeomorphic to either K5 or K3,3
    let mut to_remove = BTreeSet::new();
    let nodes = ret_undir.node_indices().collect::<Vec<_>>();

    for node in nodes {
        let neis = ret_undir
            .neighbors(node)
            .filter(|to| !to_remove.contains(to))
            .collect::<Vec<_>>();

        if neis.len() == 0 {
            to_remove.insert(node);
            continue;
        }

        if neis.len() != 2 {
            continue;
        }

        ret_undir.add_edge(neis[0], neis[1], EdgeLabel::Real);
        to_remove.insert(node);
    }

    for &node in to_remove.iter().rev() {
        ret_undir.remove_node(node);
    }

    let k_5 = UnGraph::from_edges(&[
        (0, 1, EdgeLabel::Real),
        (0, 2, EdgeLabel::Real),
        (0, 3, EdgeLabel::Real),
        (0, 4, EdgeLabel::Real),
        (1, 2, EdgeLabel::Real),
        (1, 3, EdgeLabel::Real),
        (1, 4, EdgeLabel::Real),
        (2, 3, EdgeLabel::Real),
        (2, 4, EdgeLabel::Real),
        (3, 4, EdgeLabel::Real),
    ]);
    let k_33 = UnGraph::from_edges(&[
        (0, 3, EdgeLabel::Real),
        (0, 4, EdgeLabel::Real),
        (0, 5, EdgeLabel::Real),
        (1, 3, EdgeLabel::Real),
        (1, 4, EdgeLabel::Real),
        (1, 5, EdgeLabel::Real),
        (2, 3, EdgeLabel::Real),
        (2, 4, EdgeLabel::Real),
        (2, 5, EdgeLabel::Real),
    ]);

    assert!(
        is_isomorphic(&ret_undir, &k_5) || is_isomorphic(&ret_undir, &k_33),
        "The resulting graph is not homeomorphic to K5 or K33"
    );

    ret
}
