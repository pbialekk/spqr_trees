use crate::{EdgeLabel, UnGraph};
use petgraph::visit::{IntoNodeReferences, NodeIndexable};

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

fn dfs(
    graph: &UnGraph,
    u: usize,
    parent: Option<usize>,
    time: &mut usize,
    preorder: &mut [usize],
    vertex_stack: &mut Vec<usize>,
    blocks: &mut Vec<Vec<usize>>,
    is_cut: &mut [bool],
) -> usize {
    preorder[u] = *time;
    *time += 1;
    let mut low = preorder[u];
    let mut is_potential_cut = parent.is_some();
    vertex_stack.push(u);

    for v in graph.neighbors(graph.from_index(u)).map(|n| n.index()) {
        if preorder[v] == usize::MAX {
            let stack_len = vertex_stack.len();
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
            low = low.min(low_v);
            if low_v >= preorder[u] && is_potential_cut {
                is_cut[u] = true;
                let mut block = vertex_stack[stack_len..].to_vec();
                block.push(u);
                vertex_stack.truncate(stack_len);
                blocks.push(block);
            }
            is_potential_cut = true;
        } else if Some(v) != parent {
            low = low.min(preorder[v]);
        }
    }
    low
}

/// Returns the biconnected components (blocks) of the graph and vector of block ids adjacent to each vertex.
/// Each block is a set of vertices that are biconnected.
pub fn get_block_cut_tree(graph: &UnGraph) -> BlockCutTree {
    let graph_size = graph.node_references().size_hint().0;
    let mut time = 0;
    let mut preorder = vec![usize::MAX; graph_size];
    let mut vertex_stack = Vec::with_capacity(graph_size);
    let mut is_cut = vec![false; graph_size];
    let mut blocks = Vec::new();

    for u in graph.node_indices() {
        let idx = u.index();
        if preorder[idx] == usize::MAX {
            dfs(
                graph,
                idx,
                None,
                &mut time,
                &mut preorder,
                &mut vertex_stack,
                &mut blocks,
                &mut is_cut,
            );
            if !vertex_stack.is_empty() {
                blocks.push(vertex_stack.clone());
                vertex_stack.clear();
            }
        }
    }

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
        for &u in block {
            block_graph.add_node(u.try_into().unwrap());
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
                .add_node(block_cut_tree.node_to_id[u].try_into().unwrap())
                .index();
            block_cut_tree.cut_count += 1;
        }
    }

    // Add edges between blocks and cut vertices
    for (i, block) in blocks.iter().enumerate() {
        for &u in block {
            if is_cut[u] {
                block_cut_tree.graph.add_edge(
                    block_cut_tree.graph.from_index(i),
                    block_cut_tree
                        .graph
                        .from_index(block_cut_tree.node_to_id[u]),
                    EdgeLabel::Virtual,
                );
            }
        }
    }

    // Add edges inside blocks
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
