use hashbrown::{HashMap, HashSet};
use petgraph::algo::dijkstra;
use petgraph::graph::NodeIndex;
use petgraph::visit::IntoNodeReferences;

use crate::embedding::is_planar;
use crate::example_usages::oeip::dual_graph::get_dual_graph;
use crate::testing::grids::Point;
use crate::{
    EdgeLabel, UnGraph, spqr_blocks::outside_structures::SPQRTree, spqr_tree::get_spqr_tree,
    triconnected_blocks::outside_structures::ComponentType,
};

/// Solves the Optimal Edge Insertion Problem (OEIP) for a given biconnected planar graph.
///
/// ## Statement:
/// The Optimal Edge Insertion Problem (OEIP) is the problem of inserting an edge `(u, v)` into a biconnected planar graph
/// such that the number of crossings is minimized.
///
/// ## Prerequisites:
/// - input graph is biconnected and planar,
/// - you can provide arbitrary embedding of the graph as a vector of points.
///
/// ## Idea:
/// 1. Compute SPQR tree of the input graph.
/// 2. Find the shortest path between arbitrary allocation nodes of `u` and `v` in the SPQR tree.
/// 3. Delete S and P nodes from the path. You can always insert edge without crossing.
/// Leave R nodes, they are easy problems because they have only 2 embeddings.
/// 4. For each R node in the path, iteratively, expand its edges (without virtual edges of `u` and `v`).
/// + Find arbitrary embedding of the expanded graph.
/// + Construct dual graph of the expanded graph.
/// + Add two new nodes to the dual graph, one for `u` and one for `v`. Connect them to adjacent faces.
/// + Find the shortest path between `u'` and `v'` in the dual graph.
/// This is your number of crossings in this component.
/// 5. Sum up the number of crossings for all R nodes in the path.
///
/// ## Testing:
/// We only tested this algorithm on grid graphs.
/// We can easily compute the number of crossings for any pair of vertices in a grid graph by hand.
/// It is done in tests section.
///
/// ## Complexity:
/// Almost all operations are linear in the size of the input graph.
/// But finding the dual graph is `O(nlog(n))`.
/// So overall complexity is dependent of construction of the dual graph.
///
/// NOTE:
/// - SPQR construction is linear.
/// - Finding path is linear.
/// - We also expand only the necessary part of skeleton graph.
///
/// ## Reference:
/// - [Optimal Edge Insertion Problem](https://www.ac.tuwien.ac.at/files/pub/Gutwenger01.pdf)

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OptimalBlockInserter {
    /// Input graph
    graph: UnGraph,
    /// Arbitrary embedding
    points: Vec<Point>,
    /// SPQR tree of the input graph
    tree: SPQRTree,
    /// Set of vertices in each component of the SPQR tree
    component_vertex_set: Vec<HashSet<usize>>,
    /// Arbitrary allocation node for each vertex in the input graph
    first_allocation_node: Vec<usize>,
    /// Map of pairs of components to virtual edge id in the SPQR tree
    pair_of_components_to_virt_edge: HashMap<(usize, usize), usize>,
}

#[allow(dead_code)]
impl OptimalBlockInserter {
    pub fn new(graph: &UnGraph, points: Vec<Point>) -> Self {
        assert!(is_planar(graph, false).0, "Graph must be planar");

        let tree = get_spqr_tree(&graph);
        let mut component_vertex_set = vec![HashSet::new(); tree.blocks.comp.len()];
        let mut first_allocation_node = vec![None; graph.node_references().count()];
        let mut pair_of_components_to_virt_edge = HashMap::new();

        // We have traverse the SPQR tree to populate the allocation nodes.

        fn populate_allocation_info(
            tree: &SPQRTree,
            first_allocation_node: &mut Vec<Option<usize>>,
            component_vertex_set: &mut Vec<HashSet<usize>>,
            u: usize,
            parent: Option<usize>,
        ) {
            for &eid in tree.blocks.comp[u].edges.iter() {
                let (a, b) = tree.blocks.edges[eid];

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
            pair_of_components_to_virt_edge: &mut HashMap<(usize, usize), usize>,
        ) {
            let mut virt_edges = HashMap::new();
            for (i, component) in tree.blocks.comp.iter().enumerate() {
                for &eid in component.edges.iter() {
                    if virt_edges.contains_key(&eid) {
                        // add both pairs just for convenience
                        pair_of_components_to_virt_edge.insert((virt_edges[&eid], i), eid);
                        pair_of_components_to_virt_edge.insert((i, virt_edges[&eid]), eid);
                    } else {
                        virt_edges.insert(eid, i);
                    }
                }
            }
        }

        if tree.blocks.comp.len() > 0 {
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
                first_allocation_node: first_allocation_node
                    .into_iter()
                    .map(|x| x.unwrap())
                    .collect(),
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
    fn find_arbitrary_path_between_allocation_nodes(&self, u: usize, v: usize) -> Vec<usize> {
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
            false
        }

        find_path(&self.tree, start, end, None, &mut path);

        path
    }

    /// Deletes unnecessary nodes from the path between two allocation nodes.
    fn find_shortest_path_between_allocation_nodes(&self, u: usize, v: usize) -> Vec<usize> {
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
    fn delete_sp_nodes_from_path(&self, path: &Vec<usize>) -> Vec<usize> {
        let mut reduced_path = vec![];
        for &node in path.iter() {
            if self.tree.blocks.comp[node].comp_type == ComponentType::R {
                reduced_path.push(node);
            }
        }
        reduced_path
    }

    /// Returns the optimal number of crossings when inserting edge (u, v) into graph.
    pub fn oeip(&self, u: usize, v: usize) -> i32 {
        if u == v {
            return 0;
        }
        if self
            .graph
            .find_edge(NodeIndex::new(u), NodeIndex::new(v))
            .is_some()
        {
            return 0;
        }

        let path = self.find_shortest_path_between_allocation_nodes(u, v);
        let reduced_path = self.delete_sp_nodes_from_path(&path);
        if reduced_path.is_empty() {
            return 0;
        }
        let mut crossings = 0;

        // Updates list of edges of expanded skeleton graph.
        fn expand_skeleton(
            tree: &SPQRTree,
            edges: &mut Vec<usize>,
            marked_edges: &Vec<bool>,
            u: usize,
            parent: Option<usize>,
            pair_of_components_to_virt_edge: &HashMap<(usize, usize), usize>,
        ) {
            for &eid in tree.blocks.comp[u].edges.iter() {
                if !marked_edges[eid] && tree.blocks.is_real[eid] {
                    edges.push(eid);
                }
            }

            for &to in tree.adj[u].iter() {
                if Some(to) == parent {
                    continue;
                }
                // We don't want to expand marked virtual edges
                if !marked_edges[pair_of_components_to_virt_edge[&(u, to)]] {
                    expand_skeleton(
                        tree,
                        edges,
                        marked_edges,
                        to,
                        Some(u),
                        pair_of_components_to_virt_edge,
                    );
                }
            }
        }

        // Iterate through path
        for (i, node) in path.iter().enumerate() {
            if self.tree.blocks.comp[*node].comp_type != ComponentType::R {
                continue; // if deleted there were problems with prev and next
            }
            let mut edges = vec![];
            let mut u_virt_edge = None;
            let mut v_virt_edge = None;
            let mut marked_edges = vec![false; self.tree.blocks.edges.len()];

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

            expand_skeleton(
                &self.tree,
                &mut edges,
                &marked_edges,
                *node,
                None,
                &self.pair_of_components_to_virt_edge,
            );

            let mut expanded_graph = UnGraph::new_undirected();
            let mut node_to_expanded = HashMap::new();
            // Construct expanded graph
            for &eid in edges.iter() {
                let (a, b) = self.tree.blocks.edges[eid];
                for turn in [a, b] {
                    if !node_to_expanded.contains_key(&turn) {
                        let new_node = expanded_graph.add_node(turn as u32);
                        node_to_expanded.insert(turn, new_node);
                    }
                }
                expanded_graph.add_edge(
                    node_to_expanded[&a],
                    node_to_expanded[&b],
                    EdgeLabel::Real,
                );
            }

            let mut points = vec![];
            for id in expanded_graph.node_indices() {
                let point = self.points[*expanded_graph.node_weight(id).unwrap() as usize];
                points.push(point);
            }

            let mut dual_graph = get_dual_graph(&points, &expanded_graph);

            // Augment dual graph with src and dst
            let x1 = dual_graph.graph.node_count();
            let x1id = dual_graph.graph.add_node(x1 as u32);
            let x2 = dual_graph.graph.node_count();
            let x2id = dual_graph.graph.add_node(x2 as u32);

            if let Some(_u_virt_edge) = u_virt_edge {
                // Not present in skeleton
                dual_graph.graph.add_edge(
                    NodeIndex::new(dual_graph.outer_face),
                    x1id,
                    EdgeLabel::Structure,
                );
            } else {
                for (i, face) in dual_graph.faces.iter().enumerate() {
                    if face.vertices.contains(&node_to_expanded[&u].index()) {
                        dual_graph
                            .graph
                            .add_edge(NodeIndex::new(i), x1id, EdgeLabel::Structure);
                    }
                }
            }

            if let Some(_v_virt_edge) = v_virt_edge {
                // Not present in skeleton
                dual_graph.graph.add_edge(
                    NodeIndex::new(dual_graph.outer_face),
                    x2id,
                    EdgeLabel::Structure,
                );
            } else {
                for (i, face) in dual_graph.faces.iter().enumerate() {
                    if face.vertices.contains(&node_to_expanded[&v].index()) {
                        dual_graph
                            .graph
                            .add_edge(NodeIndex::new(i), x2id, EdgeLabel::Structure);
                    }
                }
            }

            // should be BFS but petgraph has dijkstra implemented ;)
            let costs = dijkstra(&dual_graph.graph, x1id, Option::from(x2id), |_| 1);
            crossings += costs.get(&x2id).unwrap() - 2; // -2  because we added edges to connect to faces
        }

        crossings
    }
}

mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::EdgeLabel;
    use crate::testing::grids::{generate_grid_graph, get_arbitrary_embedding_of_grid};

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

    #[allow(dead_code)]
    fn brute_grid_crossings(rows: usize, cols: usize, u: usize, v: usize) -> i32 {
        if u == v {
            return 0;
        }
        // get coordinates of u and v
        let (x1, y1) = ((u / cols) as i32, (u % cols) as i32);
        let (x2, y2) = ((v / cols) as i32, (v % cols) as i32);

        let d_vertical = (x1 as i32 - x2 as i32).abs();
        let d_horizontal = (y1 as i32 - y2 as i32).abs();
        let mut manhattan = d_vertical + d_horizontal;
        if d_vertical > 0 {
            manhattan -= 1;
        }
        if d_horizontal > 0 {
            manhattan -= 1;
        }
        let min_exit_vertical_1 = std::cmp::min(x1, rows as i32 - x1 - 1);
        let min_exit_horizontal_1 = std::cmp::min(y1, cols as i32 - y1 - 1);
        let min_exit_vertical_2 = std::cmp::min(x2, rows as i32 - x2 - 1);
        let min_exit_horizontal_2 = std::cmp::min(y2, cols as i32 - y2 - 1);
        let crossings = manhattan;
        crossings
            .min(min_exit_horizontal_1 + min_exit_horizontal_2)
            .min(min_exit_vertical_1 + min_exit_vertical_2)
            .min(min_exit_horizontal_1 + min_exit_vertical_2)
            .min(min_exit_horizontal_2 + min_exit_vertical_1)
    }

    #[cfg(all(test, not(debug_assertions)))]
    #[test]
    fn test_oeip() {
        for r in 2..11 {
            for c in 2..11 {
                let graph = generate_grid_graph(r, c);
                let points = get_arbitrary_embedding_of_grid(r, c);
                let block_inserter = OptimalBlockInserter::new(&graph, points);

                for u in 0..r * c {
                    for v in 0..r * c {
                        let crossings = block_inserter.oeip(u, v);
                        let brute_crossings = brute_grid_crossings(r, c, u, v);
                        assert_eq!(
                            crossings, brute_crossings,
                            "Failed for grid {}x{} with u={} and v={}",
                            r, c, u, v
                        );
                    }
                }
            }
        }
    }
}
