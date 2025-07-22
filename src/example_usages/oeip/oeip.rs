use hashbrown::{HashSet, HashMap};
use petgraph::graph::NodeIndex;
use petgraph::visit::IntoNodeReferences;
use petgraph::algo::dijkstra;

use crate::{UnGraph, spqr_blocks::outside_structures::SPQRTree, spqr_tree::get_spqr_tree, triconnected_blocks::outside_structures::ComponentType, EdgeLabel};
use crate::example_usages::oeip::dual_graph::get_dual_graph;
use crate::testing::grids::Point;

#[derive(Debug, Clone)]
pub struct OptimalBlockInserter {
    graph: UnGraph,
    points: Vec<Point>,
    tree: SPQRTree,
    component_vertex_set: Vec<HashSet<usize>>,
    first_allocation_node: Vec<usize>,
    pair_of_components_to_virt_edge: HashMap<(usize, usize), usize>,
}

impl OptimalBlockInserter {
    pub fn new(graph: &UnGraph, points: Vec<Point>) -> Self {
        let tree = get_spqr_tree(&graph);
        let mut component_vertex_set = vec![HashSet::new(); tree.triconnected_components.components.len()];
        let mut first_allocation_node= vec![None; graph.node_references().count()];
        let mut pair_of_components_to_virt_edge = HashMap::new();

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

        fn populate_virt_edge_info(
            tree: &SPQRTree,
            pair_of_components_to_virt_edge: &mut HashMap<(usize, usize), usize>
        ) {
            let mut virt_edges = HashMap::new();
            for (i, component) in tree.triconnected_components.components.iter().enumerate() {
                for &eid in component.edges.iter() {
                    if virt_edges.contains_key(&eid) {
                        pair_of_components_to_virt_edge.insert((virt_edges[&eid], i), eid);
                        pair_of_components_to_virt_edge.insert((i, virt_edges[&eid]), eid);
                    } else {
                        virt_edges.insert(eid, i);
                    }
                }
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

            populate_virt_edge_info(&tree, &mut pair_of_components_to_virt_edge);

            OptimalBlockInserter {
                graph: graph.clone(),
                points,
                tree,
                first_allocation_node: first_allocation_node.into_iter().map(|x| x.unwrap()).collect(),
                component_vertex_set,
                pair_of_components_to_virt_edge,
            }
        } else {
            OptimalBlockInserter {
                graph: graph.clone(),
                points,
                tree,
                first_allocation_node: vec![],
                component_vertex_set: vec![],
                pair_of_components_to_virt_edge: HashMap::new(),
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
        let mut reduced_path = vec![];
        for &node in path.iter() {
            if self.tree.triconnected_components.components[node].component_type == ComponentType::R {
                reduced_path.push(node);
            }

        }
        reduced_path
    }

    /// Returns the optimal number of crossings when inserting edge (u, v) into graph.
    ///
    /// Prerequisite: input graph is biconnected
    pub fn oeip(&self, u: usize, v: usize) -> i64 {
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

        // Updates list of edges of expanded skeleton graph.
        fn expand_skeleton(tree: &SPQRTree, edges: &mut Vec<usize>, marked_edges: &Vec<bool>, u: usize, parent: Option<usize>, pair_of_components_to_virt_edge: &HashMap<(usize, usize), usize>) {
            for &eid in tree.triconnected_components.components[u].edges.iter() {
                if !marked_edges[eid] && tree.triconnected_components.is_real_edge[eid] {
                    edges.push(eid);
                }
            }


            for &to in tree.adj[u].iter() {
                if Some(to) == parent {
                    continue;
                }
                if !marked_edges[pair_of_components_to_virt_edge[&(u, to)]] {
                    expand_skeleton(tree, edges, marked_edges, to, Some(u), pair_of_components_to_virt_edge);
                }
            }
        }

        for (i, node) in path.iter().enumerate() {
            if self.tree.triconnected_components.components[*node].component_type != ComponentType::R {
                continue; // if deleted there were problems with prev and next
            }
            let mut edges = vec![];
            let mut u_virt_edge = None;
            let mut v_virt_edge = None;
            let mut marked_edges = vec![false; self.tree.triconnected_components.edges.len()];

            if !self.component_vertex_set[*node].contains(&u) {
                let prev_node = path[i - 1];
                u_virt_edge = Some(self.pair_of_components_to_virt_edge[&(*node, prev_node)]);
                marked_edges[u_virt_edge.unwrap()] = true;
            }

            if !self.component_vertex_set[*node].contains(&v) {
                let next_node = path[i + 1];
                v_virt_edge = Some(self.pair_of_components_to_virt_edge[&(*node, next_node)]);
                marked_edges[v_virt_edge.unwrap()] = true;
            }

            expand_skeleton(&self.tree, &mut edges, &marked_edges, *node, None, &self.pair_of_components_to_virt_edge);
            println!("expanded edges: {:?}", edges);

            let mut expanded_graph = UnGraph::new_undirected();
            let mut node_to_expanded = HashMap::new();
            for &eid in edges.iter() {
                let (a, b) = self.tree.triconnected_components.edges[eid];
                for turn in [a, b] {
                    if !node_to_expanded.contains_key(&turn) {
                        let new_node = expanded_graph.add_node(turn as u32);
                        node_to_expanded.insert(turn, new_node);
                    }
                }
                expanded_graph.add_edge(node_to_expanded[&a], node_to_expanded[&b], EdgeLabel::Real);
            }

            let mut points = vec![]; // TODO: there is a evil bug
            for id in expanded_graph.node_indices() {
                let point = self.points[*expanded_graph.node_weight(id).unwrap() as usize];
                points.push(point);
            }

            let mut dual_graph = get_dual_graph(&points, &expanded_graph);

            // augment dual graph with src and dst
            let x1 = dual_graph.graph.node_count();
            let x1id = dual_graph.graph.add_node(x1 as u32);
            let x2 = dual_graph.graph.node_count();
            let x2id = dual_graph.graph.add_node(x2 as u32);

            if let Some(u_virt_edge) = u_virt_edge {
                dual_graph.graph.add_edge(0.into(), x1id, EdgeLabel::Structure);
            } else {
                println!("u:");
                for (i, face) in dual_graph.faces.iter().enumerate() {
                    if face.vertices.contains(&node_to_expanded[&u].index()) {
                        println!("{i}");
                        dual_graph.graph.add_edge(NodeIndex::new(i), x1id, EdgeLabel::Structure);
                    }
                }
            }

            if let Some(v_virt_edge) = v_virt_edge {
                dual_graph.graph.add_edge(0.into(), x2id, EdgeLabel::Structure);
            } else {
                println!("v:");
                for (i, face) in dual_graph.faces.iter().enumerate() {
                    if face.vertices.contains(&node_to_expanded[&v].index()) {
                        println!("{i}");
                        dual_graph.graph.add_edge(NodeIndex::new(i), x2id, EdgeLabel::Structure);
                    }
                }
            }

            println!("{:?}", node_to_expanded);
            println!("{:?}", dual_graph.faces);


            // TODO: get rid of dijkstra
            let costs = dijkstra(&dual_graph.graph, x1id, Option::from(x2id), |_| 1);
            crossings += costs.get(&x2id).unwrap() - 2;
        }

        crossings
    }

}

mod tests {
    use crate::EdgeLabel;
    use crate::testing::grids::{generate_grid_graph, get_arbitrary_embedding_of_grid};
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

        let block_inserter = OptimalBlockInserter::new(&graph, vec![]); // do not care about points in this test
        let path = block_inserter.find_shortest_path_between_allocation_nodes(0, 7);
        assert_eq!(path.len(), 3);
        let reduced_path = block_inserter.delete_sp_nodes_from_path(&path);
        assert_eq!(reduced_path.len(), 0);
    }

    #[test]
    fn test_oeip() { // TODO: test exhaustively grid
        let graph = generate_grid_graph(5, 5);
        let points = get_arbitrary_embedding_of_grid(5, 5);

        let block_inserter = OptimalBlockInserter::new(&graph, points);
        let crossings = block_inserter.oeip(0, 6);
        assert_eq!(crossings, 0);
    }
}
