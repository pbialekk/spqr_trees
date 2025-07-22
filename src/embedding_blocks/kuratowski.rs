use crate::{EdgeLabel, UnGraph, embedding::is_planar, types::DiGraph};

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

    ret
}
