use crate::{EdgeLabel, UnGraph};
use petgraph::graph::NodeIndex;
use petgraph::visit::{EdgeRef, IntoNodeReferences, NodeIndexable};

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
    /// If node is a block, it will be mapped to its block id.
    pub node_to_id: Vec<usize>,
}

impl BlockCutTree {}

/// Returns the lowest preorder reachable from subtree of u [lowpoint].
fn dfs(
    graph: &UnGraph,
    u: usize,
    parent: Option<usize>,
    time: &mut usize,
    preorder: &mut [usize],
    vertex_stack: &mut Vec<usize>,
    // block is defined by set of edges, this way we avoid problem with cut vertices multi membership
    blocks: &mut Vec<Vec<(usize, usize)>>,
    is_cut: &mut [bool],
) -> usize {
    preorder[u] = *time;
    *time += 1;
    let mut low = preorder[u];
    vertex_stack.push(u);

    // process all neighbors of u to get true lowpoint of u
    for v in graph.neighbors(graph.from_index(u)).map(|n| n.index()) {
        if preorder[v] == usize::MAX {
            let low_v = dfs(
                graph,
                v,
                Some(u),
                time,
                preorder,
                vertex_stack,
                blocks,
                is_cut,
            );
            // maybe some descendant of v has lower lowpoint
            low = low.min(low_v);
        } else if Some(v) != parent {
            // back edge
            low = low.min(preorder[v]);
        }
    }



    if parent.is_some() && low >= preorder[parent.unwrap()] {
        // parent is cut vertex, unless it is root
        // TODO: it does not work, maybe just add 2 ifs above
        is_cut[u] = if parent.unwrap() == 0 {
            // root is cut vertex if it has more than one child
            graph.neighbors(graph.from_index(u)).count() > 1
        } else {
            true
        };
        let mut block = Vec::new();
        while let Some(w) = vertex_stack.pop() {
            // this looks scare, but in reality we just push all edges to block and avoid duplicates
            // by predicate :)
            let edges: Vec<(usize, usize)> = graph.edges(NodeIndex::new(w))
                .map(|e| (e.source().index(), e.target().index()))
                .filter(|(w, v)| preorder[*w] > preorder[*v])
                .collect();
            block.extend(edges);
            if w == u {
                break;
            }
        }

        blocks.push(block);
    }

    low
}

// /// Returns the biconnected components (blocks) of the graph and vector of block ids adjacent to each vertex.
// /// Each block is a set of vertices that are biconnected.
// pub fn get_block_cut_tree(graph: &UnGraph) -> BlockCutTree {
//     let graph_size = graph.node_references().size_hint().0;
//     let mut time = 0;
//     let mut preorder = vec![usize::MAX; graph_size];
//     let mut vertex_stack = Vec::with_capacity(graph_size);
//     let mut is_cut = vec![false; graph_size];
//     let mut blocks = Vec::new();
//
//     for u in graph.node_indices() {
//         let idx = u.index();
//         if preorder[idx] == usize::MAX {
//             dfs(
//                 graph,
//                 idx,
//                 None,
//                 &mut time,
//                 &mut preorder,
//                 &mut vertex_stack,
//                 &mut blocks,
//                 &mut is_cut,
//             );
//         }
//     }
//
//     let mut block_cut_tree = BlockCutTree {
//         block_count: blocks.len(),
//         cut_count: 0,
//         blocks: Vec::with_capacity(blocks.len()),
//         graph: UnGraph::new_undirected(),
//         node_to_id: vec![0; graph_size],
//     };
//
//     // Add blocks as nodes
//     for (i, block) in blocks.iter().enumerate() {
//         let mut block_graph = UnGraph::new_undirected();
//         for &u in block {
//             block_graph.add_node(u.try_into().unwrap());
//             block_cut_tree.node_to_id[u] = i;
//         }
//         block_cut_tree.graph.add_node(i.try_into().unwrap());
//         block_cut_tree.blocks.push(block_graph);
//     }
//
//     // Add cut vertices as nodes
//     for u in graph.node_indices().map(|n| n.index()) {
//         if is_cut[u] {
//             block_cut_tree.node_to_id[u] = block_cut_tree
//                 .graph
//                 .add_node(block_cut_tree.node_to_id[u].try_into().unwrap())
//                 .index();
//             block_cut_tree.cut_count += 1;
//         }
//     }
//
//     // Add edges between blocks and cut vertices
//     for (i, block) in blocks.iter().enumerate() {
//         for &u in block {
//             if is_cut[u] {
//                 block_cut_tree.graph.add_edge(
//                     block_cut_tree.graph.from_index(i),
//                     block_cut_tree
//                         .graph
//                         .from_index(block_cut_tree.node_to_id[u]),
//                     EdgeLabel::Virtual,
//                 );
//             }
//         }
//     }
//
//     // Add edges inside blocks
//     let mut inside_block = vec![false; graph_size];
//     let mut inside_block_id = vec![0; graph_size];
//     for (i, block) in blocks.iter().enumerate() {
//         for (j, &u) in block.iter().enumerate() {
//             inside_block[u] = true;
//             inside_block_id[u] = j;
//         }
//         let mut edges_to_add = Vec::new();
//         for &u in block {
//             for v in graph.neighbors(graph.from_index(u)).map(|n| n.index()) {
//                 if inside_block[v] && u < v {
//                     let u_idx = block_cut_tree.blocks[i].from_index(inside_block_id[u]);
//                     let v_idx = block_cut_tree.blocks[i].from_index(inside_block_id[v]);
//                     edges_to_add.push((u_idx, v_idx));
//                 }
//             }
//         }
//         for (u_idx, v_idx) in edges_to_add {
//             block_cut_tree.blocks[i].add_edge(u_idx, v_idx, EdgeLabel::Real);
//         }
//         for &u in block {
//             inside_block[u] = false;
//         }
//     }
//
//     block_cut_tree
// }
#[cfg(test)]
mod dfs_tests {
    use super::*;
    use crate::types::UnGraph;

    #[test]
    fn test_dfs_single_edge() {
        let mut g = UnGraph::new_undirected();
        let a = g.add_node(0);
        let b = g.add_node(1);
        g.add_edge(a, b, EdgeLabel::Real);

        let mut time = 0;
        let mut preorder = vec![usize::MAX; 2];
        let mut vertex_stack = Vec::new();
        let mut blocks = Vec::new();
        let mut is_cut = vec![false; 2];

        dfs(&g, 0, None, &mut time, &mut preorder, &mut vertex_stack, &mut blocks, &mut is_cut);

        assert_eq!(is_cut, vec![false, false]);
        // I take advantage of internal indices of petgraph
        assert_eq!(blocks, vec![vec![(1, 0)]]);

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

        let mut time = 0;
        let mut preorder = vec![usize::MAX; 3];
        let mut vertex_stack = Vec::new();
        let mut blocks = Vec::new();
        let mut is_cut = vec![false; 3];

        dfs(&g, 0, None, &mut time, &mut preorder, &mut vertex_stack, &mut blocks, &mut is_cut);

        assert_eq!(is_cut, vec![false, false, false]);
        assert_eq!(blocks, vec![vec![(2, 1), (2, 0), (1, 0)]]);
    }

    #[test]
    fn test_dfs_with_cut_vertex() {
        let mut g = UnGraph::new_undirected();
        let a = g.add_node(0);
        let b = g.add_node(1);
        let c = g.add_node(2);
        g.add_edge(a, b, EdgeLabel::Real);
        g.add_edge(b, c, EdgeLabel::Real);

        let mut time = 0;
        let mut preorder = vec![usize::MAX; 3];
        let mut vertex_stack = Vec::new();
        let mut blocks = Vec::new();
        let mut is_cut = vec![false; 3];

        dfs(&g, 0, None, &mut time, &mut preorder, &mut vertex_stack, &mut blocks, &mut is_cut);

        assert_eq!(is_cut, vec![false, true, false]);
        println!("{:?}", vertex_stack);
        println!("{:?}", blocks);
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn test_dfs_root_cut_vertex() {
        let mut g = UnGraph::new_undirected();
        let a = g.add_node(0);
        let b = g.add_node(1);
        let c = g.add_node(2);
        g.add_edge(a, b, EdgeLabel::Real);
        g.add_edge(a, c, EdgeLabel::Real);

        let mut time = 0;
        let mut preorder = vec![usize::MAX; 3];
        let mut vertex_stack = Vec::new();
        let mut blocks = Vec::new();
        let mut is_cut = vec![false; 3];

        dfs(&g, 0, None, &mut time, &mut preorder, &mut vertex_stack, &mut blocks, &mut is_cut);

        assert_eq!(is_cut, vec![true, false, false]);
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