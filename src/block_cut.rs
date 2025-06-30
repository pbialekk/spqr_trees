use crate::{EdgeLabel, UnGraph};
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::visit::{EdgeRef, NodeIndexable};
use hashbrown::HashSet;
use radsort;
use fixedbitset::FixedBitSet;

/// Represents the block-cut tree of a graph, containing blocks, cut vertices, and their relationships.
#[derive(Debug, Clone)]
pub struct BlockCutTree {
    /// Number of blocks in the graph.
    pub block_count: usize,
    /// Number of cut vertices in the graph.
    pub cut_count: usize,
    /// Blocks of the graph.
    pub blocks: Vec<UnGraph>,
    /// Graph of blocks and cut vertices. Blocks have numbers from 0 to block_count - 1.
    /// Cut vertices have numbers from block_count to block_count + cut_count - 1.
    pub graph: UnGraph,
    /// Maps node index to block id.
    /// If node is a cut vertex, it will be mapped to block_count + cut_id
    /// Note that cut vertex is included in multiple blocks.
    /// If node is a block, it will be mapped to its block id.
    pub node_to_id: Vec<usize>,
}

impl BlockCutTree {}

/// Returns the lowest preorder vertex reachable from subtree of u [lowpoint].
///
/// In addition, it finds biconnected components (blocks) and cut vertices.
///
/// Based on [Tarjan & Hopcroft algorithm](https://en.wikipedia.org/wiki/Biconnected_component).
///
/// # Warning
/// <div class="warning">
///
/// - Graph must be connected, otherwise you will get wrong only first BC tree not the forest.
///
/// </div>
fn dfs(
    graph: &UnGraph,
    // NodeIndex not label!!!
    u: usize,
    parent: Option<usize>,
    time: &mut usize,
    preorder: &mut [usize],
    visited_edges: &mut FixedBitSet,
    edge_stack: &mut Vec<usize>,
    // block is defined by set of edges, this way we avoid problem with cut vertices multi membership
    blocks: &mut Vec<Vec<usize>>,
    is_cut: &mut [bool],
) -> usize {
    preorder[u] = *time;
    *time += 1;
    let mut low = preorder[u];
    let mut children = 0;


    // process all edges of u to get true lowpoint of u
    for e in graph.edges(NodeIndex::new(u)) {
        let v = e.target().index();
        if preorder[v] == usize::MAX {
            // v is not visited yet
            visited_edges.set(e.id().index(), true);
            children += 1;

            let stack_len = edge_stack.len();
            edge_stack.push(e.id().index());

            let low_v = dfs(
                graph,
                v,
                Some(u),
                time,
                preorder,
                visited_edges,
                edge_stack,
                blocks,
                is_cut,
            );

            // maybe some descendant of v has lower lowpoint
            low = low.min(low_v);
            if low_v >= preorder[u] {
                // u is a cut vertex or root in both cases we need to process the block
                is_cut[u] = parent.is_some(); // we are certain that u is a cut vertex
                // by nature of DFS, all edges of biconnected component are on the stack
                let block = edge_stack[stack_len..].to_vec();
                edge_stack.truncate(stack_len);
                blocks.push(block);

            }
        } else if preorder[v] < preorder[u] && !visited_edges.contains(e.id().index()) {
            edge_stack.push(e.id().index());
            visited_edges.set(e.id().index(), true);
            low = low.min(preorder[v]);
        }

        // remember to check if root is a cut vertex
        if parent.is_none() && children > 1 {
            is_cut[u] = true;
        }
    }

    low
}

/// Returns the biconnected components (blocks) of the graph and vector of block ids adjacent to each vertex.
/// Each block is a set of vertices that are biconnected.
/// Based on [Tarjan & Hopcroft algorithm](https://en.wikipedia.org/wiki/Biconnected_component).
///
/// # Warning
/// <div class="warning">
///
/// - We consider graph with one vertex and no edges as 1 biconnected component.
/// - Graph must be connected, otherwise you will get wrong only first BC tree not the forest.
///
/// </div>
pub fn get_block_cut_tree(graph: &UnGraph) -> BlockCutTree {
    let graph_size = graph.node_count();
    let mut time = 0;
    let mut preorder = vec![usize::MAX; graph_size];
    let mut visited_edges = FixedBitSet::with_capacity(graph.edge_count());
    let mut edge_stack = Vec::with_capacity(graph.edge_count());
    let mut is_cut = vec![false; graph_size];
    let mut blocks = Vec::new();

    if graph_size == 1 && graph.edge_count() == 0 {
        // TODO: hardcode BC Tree
    }

    dfs(
        graph,
        0, // arbitrary root
        None,
        &mut time,
        &mut preorder,
        &mut visited_edges,
        &mut edge_stack,
        &mut blocks,
        &mut is_cut,
    );

    let mut block_cut_tree = BlockCutTree {
        block_count: blocks.len(),
        cut_count: 0,
        blocks: Vec::with_capacity(blocks.len()),
        graph: UnGraph::new_undirected(),
        node_to_id: vec![0; graph_size],
    };

    // Add blocks as nodes
    for (i, block) in blocks.iter().enumerate() {
        let mut block_graph = UnGraph::new_undirected();

        // Create a set of vertices indices
        let mut block_vertex_set = HashSet::new();
        for &u in block {
            let (v, w) = graph
                .edge_endpoints(EdgeIndex::new(u))
                .expect("Edge endpoints should exist");
            let v_idx = v.index();
            let w_idx = w.index();
            block_vertex_set.extend([v_idx, w_idx]);
        }

        // Sort them with linear sort to maintain labels & indices relation
        let mut block_vertices: Vec<usize> = block_vertex_set.into_iter().collect();
        radsort::sort(&mut block_vertices);

        // And just insert labels to the block graph
        for u in block_vertices {
            let label = graph.node_weight(NodeIndex::new(u)).unwrap().clone();
            block_graph.add_node(label);
            block_cut_tree.node_to_id[u] = i;
        }
        block_cut_tree.graph.add_node(i.try_into().unwrap());
        block_cut_tree.blocks.push(block_graph);
    }

    // Add cut vertices as nodes
    for u in graph.node_indices().map(|n| n.index()) {
        if is_cut[u] {
            block_cut_tree.node_to_id[u] = block_cut_tree
                .graph
                .add_node(graph.node_weight(NodeIndex::new(u)).unwrap().clone())
                .index();
            block_cut_tree.cut_count += 1;
        }
    }

    // Add edges between blocks and cut vertices
    // TODO: block is list of edges not vertices
    for (i, block) in blocks.iter().enumerate() {
        for &u in block {
            if is_cut[u] {
                block_cut_tree.graph.add_edge(
                    block_cut_tree.graph.from_index(i),
                    block_cut_tree
                        .graph
                        .from_index(block_cut_tree.node_to_id[u]),
                    EdgeLabel::Structure,
                );
            }
        }
    }

    // Add edges inside blocks
    // TODO: left there
    let mut inside_block = vec![false; graph_size];
    let mut inside_block_id = vec![0; graph_size];
    for (i, block) in blocks.iter().enumerate() {
        for (j, &u) in block.iter().enumerate() {
            inside_block[u] = true;
            inside_block_id[u] = j;
        }
        let mut edges_to_add = Vec::new();
        for &u in block {
            for v in graph.neighbors(graph.from_index(u)).map(|n| n.index()) {
                if inside_block[v] && u < v {
                    let u_idx = block_cut_tree.blocks[i].from_index(inside_block_id[u]);
                    let v_idx = block_cut_tree.blocks[i].from_index(inside_block_id[v]);
                    edges_to_add.push((u_idx, v_idx));
                }
            }
        }
        for (u_idx, v_idx) in edges_to_add {
            block_cut_tree.blocks[i].add_edge(u_idx, v_idx, EdgeLabel::Real);
        }
        for &u in block {
            inside_block[u] = false;
        }
    }

    block_cut_tree
}
#[cfg(test)]
mod dfs_tests {
    use super::*;
    use crate::types::UnGraph;

    fn run_dfs(g: &UnGraph, start: usize) -> (Vec<bool>, Vec<Vec<usize>>, Vec<usize>) {
        let mut time = 0;
        let mut preorder = vec![usize::MAX; g.node_count()];
        let mut visited_edges = FixedBitSet::with_capacity(g.edge_count());
        let mut edge_stack = Vec::new();
        let mut blocks = Vec::new();
        let mut is_cut = vec![false; g.node_count()];
        dfs(
            g,
            start,
            None,
            &mut time,
            &mut preorder,
            &mut visited_edges,
            &mut edge_stack,
            &mut blocks,
            &mut is_cut,
        );
        (is_cut, blocks, preorder)
    }

    fn assert_dfs(
        g: &UnGraph,
        start: usize,
        expected_is_cut: &[bool],
        expected_blocks: &mut [Vec<usize>],
    ) {
        let (is_cut, mut blocks, _) = run_dfs(g, start);
        // easier to test when sorted
        for block in &mut blocks {
            block.sort();
        }
        for block in &mut *expected_blocks {
            block.sort();
        }
        blocks.sort();
        expected_blocks.sort();
        assert_eq!(is_cut, expected_is_cut);
        assert_eq!(blocks, expected_blocks);
    }

    #[test]
    fn test_dfs_single_edge() {
        let mut g = UnGraph::new_undirected();
        let a = g.add_node(0);
        let b = g.add_node(1);
        g.add_edge(a, b, EdgeLabel::Real);
        assert_dfs(&g, 0, &[false, false], &mut [vec![0]]);
    }

    #[test]
    fn test_dfs_triangle() {
        let mut g = UnGraph::new_undirected();
        let a = g.add_node(0);
        let b = g.add_node(1);
        let c = g.add_node(2);
        g.add_edge(a, b, EdgeLabel::Real);
        g.add_edge(b, c, EdgeLabel::Real);
        g.add_edge(c, a, EdgeLabel::Real);
        assert_dfs(&g, 0, &[false, false, false], &mut [vec![0, 1, 2]]);
    }

    #[test]
    fn test_dfs_with_cut_vertex() {
        let mut g = UnGraph::new_undirected();
        let a = g.add_node(0);
        let b = g.add_node(1);
        let c = g.add_node(2);
        g.add_edge(a, b, EdgeLabel::Real);
        g.add_edge(b, c, EdgeLabel::Real);

        assert_dfs(&g, 0, &[false, true, false], &mut [vec![0], vec![1]]);
    }

    #[test]
    fn test_dfs_root_cut_vertex() {
        let mut g = UnGraph::new_undirected();
        let a = g.add_node(0);
        let b = g.add_node(1);
        let c = g.add_node(2);
        g.add_edge(a, b, EdgeLabel::Real);
        g.add_edge(a, c, EdgeLabel::Real);

        assert_dfs(&g, 0, &[true, false, false], &mut [vec![0], vec![1]]);
    }

    #[test]
    fn test_dfs_complex_graph() {
        let mut g = UnGraph::new_undirected();
        let a = g.add_node(0);
        let b = g.add_node(1);
        let c = g.add_node(2);
        let d = g.add_node(3);
        let e = g.add_node(4);
        let f = g.add_node(5);
        g.add_edge(a, b, EdgeLabel::Real);
        g.add_edge(b, c, EdgeLabel::Real);
        g.add_edge(c, a, EdgeLabel::Real);
        g.add_edge(d, e, EdgeLabel::Real);
        g.add_edge(e, f, EdgeLabel::Real);
        g.add_edge(f, d, EdgeLabel::Real);
        g.add_edge(a, d, EdgeLabel::Real);
        // 1----\        /---- 4
        // |     0 ---- 3      |
        // 2----/        \---- 5

        assert_dfs(
            &g,
            0,
            &[true, false, false, true, false, false],
            &mut [vec![0, 1, 2], vec![3, 4, 5], vec![6]],
        );
    }

    #[test]
    fn test_dfs_multigraph() {
        let mut g = UnGraph::new_undirected();
        let a = g.add_node(0);
        let b = g.add_node(1);
        let c = g.add_node(2);
        let d = g.add_node(3);
        g.add_edge(a, b, EdgeLabel::Real);
        g.add_edge(a, b, EdgeLabel::Real); // parallel edge
        g.add_edge(a, b ,EdgeLabel::Real); // parallel edge
        g.add_edge(b, c, EdgeLabel::Real);
        g.add_edge(c, d, EdgeLabel::Real);
        g.add_edge(d, b, EdgeLabel::Real);
        // 0 =3= 1 -- 2
        //        \   |
        //         \  |
        //           3

        assert_dfs(&g, 0, &[false, true, false, false],
                   &mut [vec![0, 1, 2], vec![3, 4, 5]]);
    }
}

// #[cfg(test)]
// mod bc_tests {
//     use super::*;
//     use petgraph::graph::UnGraph;
//
//     #[test]
//     fn test_bc_single_edge() {
//         let mut g = UnGraph::new_undirected();
//         let a = g.add_node(0);
//         let b = g.add_node(1);
//         g.add_edge(a, b, EdgeLabel::Real);
//
//         let bct = get_block_cut_tree(&g);
//         assert_eq!(bct.block_count, 1);
//         assert_eq!(bct.cut_count, 0);
//         assert_eq!(bct.blocks.len(), 1);
//     }
//
//     #[test]
//     fn test_bc_triangle() {
//         let mut g = UnGraph::new_undirected();
//         let a = g.add_node(0);
//         let b = g.add_node(1);
//         let c = g.add_node(2);
//         g.add_edge(a, b, EdgeLabel::Real);
//         g.add_edge(b, c, EdgeLabel::Real);
//         g.add_edge(c, a, EdgeLabel::Real);
//
//         let bct = get_block_cut_tree(&g);
//         assert_eq!(bct.block_count, 1);
//         assert_eq!(bct.cut_count, 0);
//         assert_eq!(bct.blocks.len(), 1);
//     }
//
//     #[test]
//     fn test_bc_cut_vertex() {
//         let mut g = UnGraph::new_undirected();
//         let a = g.add_node(0);
//         let b = g.add_node(1);
//         let c = g.add_node(2);
//         g.add_edge(a, b, EdgeLabel::Real);
//         g.add_edge(b, c, EdgeLabel::Real);
//
//         let bct = get_block_cut_tree(&g);
//         assert_eq!(bct.cut_count, 1);
//         assert_eq!(bct.block_count, 2);
//         assert_eq!(bct.blocks.len(), 2);
//     }
//
//     #[test]
//     fn test_bc_root_cut_vertex() {
//         let mut g = UnGraph::new_undirected();
//         let a = g.add_node(0);
//         let b = g.add_node(1);
//         let c = g.add_node(2);
//         g.add_edge(a, b, EdgeLabel::Real);
//         g.add_edge(a, c, EdgeLabel::Real);
//
//         let bct = get_block_cut_tree(&g);
//         assert_eq!(bct.cut_count, 1);
//         assert_eq!(bct.block_count, 2);
//         assert_eq!(bct.blocks.len(), 2);
//     }
// }
