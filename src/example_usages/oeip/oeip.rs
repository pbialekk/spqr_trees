use hashbrown::HashSet;
use petgraph::graph::NodeIndex;
use petgraph::visit::IntoNodeReferences;

use crate::{
    UnGraph, spqr_blocks::outside_structures::SPQRTree, spqr_tree::get_spqr_tree,
    triconnected_blocks::outside_structures::ComponentType,
};

#[derive(Debug, Clone)]
pub struct OptimalBlockInserter {
    graph: UnGraph,
    tree: SPQRTree,
    component_vertex_set: Vec<HashSet<usize>>,
    first_allocation_node: Vec<usize>,
}

impl OptimalBlockInserter {
    pub fn new(graph: &UnGraph) -> Self {
        let tree = get_spqr_tree(&graph);
        let mut component_vertex_set = vec![HashSet::new(); tree.triconnected_components.components.len()];
        let mut first_allocation_node= vec![None; graph.node_references().count()];

        // We have traverse the SPQR tree to populate the allocation nodes.

        fn populate_allocation_info(
            tree: &SPQRTree,
            first_allocation_node: &mut Vec<Option<usize>>,
            component_vertex_set: &mut Vec<HashSet<usize>>,
            u: usize,
            parent: Option<usize>,
        ) {
            for &eid in tree.triconnected_components.components[u].edges.iter() {

                let (a, b) = tree.triconnected_components.edges[eid];

                for turn in [a, b] {
                    if first_allocation_node[turn].is_none() {
                        first_allocation_node[turn] = Some(u);
                    }
                    component_vertex_set[u].insert(turn);
                }
            }

            for &to in tree.adj[u].iter() {
                if Some(to) == parent {
                    continue;
                }
                populate_allocation_info(
                    tree,
                    first_allocation_node,
                    component_vertex_set,
                    to,
                    Some(u),
                );
            }
        }

        if tree.triconnected_components.components.len() > 0 {
            populate_allocation_info(
                &tree,
                &mut first_allocation_node,
                &mut component_vertex_set,
                0,
                None,
            );

            OptimalBlockInserter {
                graph: graph.clone(),
                tree,
                first_allocation_node: first_allocation_node.into_iter().map(|x| x.unwrap()).collect(),
                component_vertex_set
            }
        } else {
            OptimalBlockInserter {
                graph: graph.clone(),
                tree,
                first_allocation_node: vec![],
                component_vertex_set: vec![],
            }
        }
    }

    /// Finds arbitrary path between two allocation nodes in the SPQR tree.
    ///
    /// Path is unique, because we are dealing with a tree, but we can choose multiple allocation nodes pairs.
    fn find_arbitrary_path_between_allocation_nodes(
        &self,
        u: usize,
        v: usize,
    ) -> Vec<usize> {
        let mut path = vec![];
        let start = self.first_allocation_node[u];
        let end = self.first_allocation_node[v];
        fn find_path(
            tree: &SPQRTree,
            w: usize,
            end: usize,
            parent: Option<usize>,
            path: &mut Vec<usize>,
        ) -> bool {
            path.push(w);
            if w == end {
                return true;
            }
            for &to in tree.adj[w].iter() {
                if Some(to) == parent {
                    continue;
                }
                if !find_path(tree, to, end, Some(w), path) {
                    path.pop();
                } else {
                    return true;
                }
            }
            return false
        }

        find_path(&self.tree, start, end, None, &mut path);

        path
    }

    /// Deletes unnecessary nodes from the path between two allocation nodes.
    fn find_shortest_path_between_allocation_nodes(
        &self,
        u: usize,
        v: usize,
    ) -> Vec<usize> {
        let mut path = self.find_arbitrary_path_between_allocation_nodes(u, v);
        path.reverse();
        while path.len() > 1 {
            if self.component_vertex_set[path[path.len() - 2]].contains(&u) {
                path.pop();
            } else {
                break;
            }
        }
        path.reverse();
        while path.len() > 1 {
            if self.component_vertex_set[path[path.len() - 2]].contains(&v) {
                path.pop();
            } else {
                break;
            }
        }

        path
    }

    /// Deletes S and P nodes from the path between two allocation nodes.
    ///
    /// They are not relevant.
    fn delete_sp_nodes_from_path(
        &self,
        path: &Vec<usize>,
    ) -> Vec<usize> {
        let mut recuded_path = vec![];
        for &node in path.iter() {
            if self.tree.triconnected_components.components[node].component_type == ComponentType::R {
                recuded_path.push(node);
            }

        }
        recuded_path
    }

    /// Returns the optimal number of crossings when inserting edge (u, v) into graph.
    ///
    /// Prerequisite: input graph is biconnected
    pub fn oeip(&self, u: usize, v: usize) -> usize {
        if u == v {
            return 0;
        }
        if self.graph.find_edge(NodeIndex::new(u), NodeIndex::new(v)).is_some() {
            return 0;
        }

        let path = self.find_shortest_path_between_allocation_nodes(u, v);
        let reduced_path = self.delete_sp_nodes_from_path(&path);
        if reduced_path.is_empty() {
            return 0;
        }
        let mut crossings = 0;

        for node in reduced_path.iter() {

        }

        crossings
    }

}

mod tests {
    use crate::EdgeLabel;
    use super::*;

    #[test]
    fn test_find_shortest_path_between_allocation_nodes() {
        let mut graph = UnGraph::new_undirected();
        graph.add_node(0);
        graph.add_node(1);
        graph.add_node(2);
        graph.add_node(3);
        graph.add_node(4);
        graph.add_node(5);
        graph.add_node(6);
        graph.add_node(7);
        graph.add_edge(0.into(), 1.into(), EdgeLabel::Real);
        graph.add_edge(1.into(), 2.into(), EdgeLabel::Real);
        graph.add_edge(2.into(), 3.into(), EdgeLabel::Real);
        graph.add_edge(3.into(), 0.into(), EdgeLabel::Real);
        graph.add_edge(0.into(), 4.into(), EdgeLabel::Real);
        graph.add_edge(3.into(), 4.into(), EdgeLabel::Real);
        graph.add_edge(0.into(), 5.into(), EdgeLabel::Real);
        graph.add_edge(1.into(), 5.into(), EdgeLabel::Real);
        graph.add_edge(1.into(), 6.into(), EdgeLabel::Real);
        graph.add_edge(2.into(), 6.into(), EdgeLabel::Real);
        graph.add_edge(2.into(), 7.into(), EdgeLabel::Real);
        graph.add_edge(3.into(), 7.into(), EdgeLabel::Real);

        let block_inserter = OptimalBlockInserter::new(&graph);
        let path = block_inserter.find_shortest_path_between_allocation_nodes(0, 7);
        assert_eq!(path.len(), 3);
        let reduced_path = block_inserter.delete_sp_nodes_from_path(&path);
        assert_eq!(reduced_path.len(), 0);
    }
}
