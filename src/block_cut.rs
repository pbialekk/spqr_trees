use embed_doc_image::embed_doc_image;
use crate::{DFSEdgeLabel, EdgeLabel, UnGraph};
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::visit::{EdgeRef, NodeIndexable};
use hashbrown::{HashSet};
use radsort;

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
    /// This map goes from original graph internal indices to block-cut tree skeleton indices.
    pub node_to_id: Vec<usize>,
    ///  Labels of edges
    pub edge_labels: Vec<DFSEdgeLabel>,
    /// Preorder
    pub preorder: Vec<usize>,
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
/// - Graph must be connected, otherwise you will get only first BC tree not the forest.
///
/// </div>
fn dfs(
    graph: &UnGraph,
    // NodeIndex not label!!!
    u: usize,
    parent: Option<usize>,
    time: &mut usize,
    preorder: &mut [usize],
    edge_labels: &mut [DFSEdgeLabel],
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
            edge_labels[e.id().index()] = DFSEdgeLabel::Tree;
            children += 1;

            let stack_len = edge_stack.len();
            edge_stack.push(e.id().index());

            let low_v = dfs(
                graph,
                v,
                Some(u),
                time,
                preorder,
                edge_labels,
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
        } else if preorder[v] < preorder[u] && edge_labels[e.id().index()] == DFSEdgeLabel::Unvisited {
            // may be parallel edge or back edge
            edge_stack.push(e.id().index());
            edge_labels[e.id().index()] = DFSEdgeLabel::Back;
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
/// - Graph must be connected, otherwise you will get  only first BC tree not the forest.
/// - We are assuming that graph is simple.
///
/// </div>
///
/// # Basic idea behind algorithm
/// With DFS we can identify articulations (cut vertices).
/// We do this by checking if the lowpoint of a child is greater than or equal to the preorder of the parent.
/// Then we take advantage of DFS traversal to find biconnected components.
/// We keep stack of visited edges and when we find a cut vertex or a root, we simply pop edges from the stack.
/// These steps are done in the `dfs` function above in source code.
/// In `get_block_cut_tree` we collect the blocks and cut vertices into a `BlockCutTree` structure.
///
/// # Warning
/// <div class="warning">
///
/// - Internal indices of nodes may not remain the same, because we create new subgraphs. But labels of nodes are preserved.
///
/// </div>
///
/// # Example
/// ```rust
/// use spqr_trees::input::from_file;
/// use spqr_trees::block_cut::{get_block_cut_tree, draw_full_block_cut_tree};
///
/// let graph = from_file("assets/bc.in");
/// let bc_tree = get_block_cut_tree(&graph);
///
/// assert_eq!(bc_tree.block_count, 7);
/// assert_eq!(bc_tree.cut_count, 4);
/// ```
///
/// # Walkthrough
/// Given graph looks like this:
///
/// ![BC_Graph][bc]
///
/// Then DFS will find articulation points (cut vertices):
///
/// ![BC_DFS][bc_dfs]
///
/// And finally we will get the block-cut tree:
///
/// ![BC_Full][bc_full]
#[embed_doc_image("bc", "assets/bc.svg")]
#[embed_doc_image("bc_dfs", "assets/bc_dfs.svg")]
#[embed_doc_image("bc_full", "assets/bc_full.svg")]

pub fn get_block_cut_tree(graph: &UnGraph) -> BlockCutTree {
    let graph_size = graph.node_count();
    let mut time = 0;
    let mut preorder = vec![usize::MAX; graph_size];
    let mut edge_labels = vec![DFSEdgeLabel::Unvisited; graph.edge_count()];
    let mut edge_stack = Vec::with_capacity(graph.edge_count());
    let mut is_cut = vec![false; graph_size];
    let mut blocks = Vec::new();

    if graph_size == 1 && graph.edge_count() == 0 {
        let mut block_cut_tree = BlockCutTree {
            block_count: 1,
            cut_count: 0,
            blocks: vec![UnGraph::new_undirected()],
            graph: UnGraph::new_undirected(),
            node_to_id: vec![0],
            edge_labels: vec![],
            preorder: vec![0],
        };

        block_cut_tree.blocks[0].add_node(graph.node_weight(NodeIndex::new(0)).unwrap().clone());
        block_cut_tree.graph.add_node(0);

        return block_cut_tree;
    }

    dfs(
        graph,
        0, // arbitrary root
        None,
        &mut time,
        &mut preorder,
        &mut edge_labels,
        &mut edge_stack,
        &mut blocks,
        &mut is_cut,
    );

    // Sets of vertices in each block
    let mut blocks_vertices_sets: Vec<HashSet<usize>> = vec![HashSet::new(); blocks.len()];

    // Map from current internal indices to new biconnected component internal indices
    let mut bicon_internal_indices: Vec<usize> = vec![0; graph_size];

    let mut block_cut_tree = BlockCutTree {
        block_count: blocks.len(),
        cut_count: 0,
        blocks: Vec::with_capacity(blocks.len()),
        graph: UnGraph::new_undirected(),
        node_to_id: vec![0; graph_size],
        edge_labels,
        preorder: preorder.clone(),
    };

    // Add blocks as nodes
    for (i, block) in blocks.iter().enumerate() {
        let mut block_graph = UnGraph::new_undirected();

        for &edge_idx in block {
            let (v, w) = graph
                .edge_endpoints(EdgeIndex::new(edge_idx))
                .expect("Edge endpoints should exist");
            let v_idx = v.index();
            let w_idx = w.index();
            blocks_vertices_sets[i].extend([v_idx, w_idx]);
        }

        // Sort them with linear sort to maintain labels and internal indices relation
        let mut block_vertices: Vec<usize> = blocks_vertices_sets[i].iter().copied().collect();
        radsort::sort(&mut block_vertices);

        // And just insert labels to the block graph
        for u in block_vertices {
            let label = graph.node_weight(NodeIndex::new(u)).unwrap().clone();
            bicon_internal_indices[u] = block_graph.add_node(label).index();
            block_cut_tree.node_to_id[u] = i;
        }

        // Add edges inside blocks
        for &edge_idx in block {
            let (v, w) = graph
                .edge_endpoints(EdgeIndex::new(edge_idx))
                .expect("Edge endpoints should exist");
            let v_idx = v.index();
            let w_idx = w.index();
            block_graph.add_edge(
                NodeIndex::new(bicon_internal_indices[v_idx]),
                NodeIndex::new(bicon_internal_indices[w_idx]),
                EdgeLabel::Real);
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
    for (i, vertex_set) in blocks_vertices_sets.iter().enumerate() {
        for &u in vertex_set {
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

    block_cut_tree
}

/// Output a skeleton of the block-cut tree in DOT format.
/// Biconnected components (blocks) are represented as green nodes labeled B_i.
/// Cut vertices are represented as red nodes with their real labels.
///
/// Intended to use with `neato`.
pub fn draw_skeleton_of_block_cut_tree(bct: &BlockCutTree) -> String {
    let mut output = String::from("graph {\n");
    // It just works
    output.push_str("  mode=sgd;\n");
    output.push_str("  maxiter=1000;\n");
    output.push_str("  node [style=filled];\n");

    // Add block nodes (green, label B_i)
    for i in 0..bct.block_count {
        output.push_str(&format!(
            "  block{} [label=\"B{}\", fillcolor=lightgreen, shape=box];\n",
            i, i
        ));
    }

    // Add cut vertex nodes (red, real labels)
    for i in 0..bct.cut_count {
        let idx = bct.block_count + i;
        let label = bct.graph.node_weight(NodeIndex::new(idx)).unwrap();
        output.push_str(&format!(
            "  cut{} [label=\"{}\", fillcolor=lightcoral, shape=circle];\n",
            idx, label
        ));
    }

    // Add edges between blocks and cut vertices
    for edge in bct.graph.edge_references() {
        let (a, b) = (edge.source().index(), edge.target().index());
        let a_str = if a < bct.block_count {
            format!("block{}", a)
        } else {
            format!("cut{}", a)
        };

        let b_str = if b < bct.block_count {
            format!("block{}", b)
        } else {
            format!("cut{}", b)
        };

        output.push_str(&format!("  {} -- {} [penwidth=2];\n", a_str, b_str));
    }

    output.push_str("}\n");
    output
}

/// It does almost exact same thing as `draw_skeleton_of_block_cut_tree`,
/// but it expands blocks into subgraphs.
///
/// Intended to use with `neato`.
pub fn draw_full_block_cut_tree(bct: &BlockCutTree) -> String {
    let mut output = String::from("graph {\n");
    // It just works for trees, draws without crossings
    output.push_str("  mode=sgd;\n");
    output.push_str("  maxiter=1000;\n");
    output.push_str("  node [style=filled, shape=circle];\n");

    // Draw each block as a cluster (lightgreen cloud)
    for (i, block) in bct.blocks.iter().enumerate() {
        output.push_str(&format!("  subgraph cluster_{} {{\n", i));
        output.push_str("    style=filled;\n    color=lightgreen;\n");
        output.push_str("    node [style=filled, fillcolor=lightblue];\n");
        // Add vertices
        for node in block.node_indices() {
            let label = block.node_weight(node).unwrap();
            output.push_str(&format!("    b_{}_{} [label=\"{}\"];\n", i, label, label));
        }
        // Add edges inside the block
        for edge in block.edge_references() {
            let (a, b) = (edge.source(), edge.target());
            let (label_a, label_b) = (
                block.node_weight(a).unwrap(),
                block.node_weight(b).unwrap(),
            );
            output.push_str(&format!(
                "    b_{}_{} -- b_{}_{};\n", i, label_a, i, label_b
            ));
        }
        output.push_str("  }\n");
    }

    // Helper
    let mut cut_vertices_labels = HashSet::new();

    // Draw cut vertices as box nodes outside clusters
    for i in 0..bct.cut_count {
        let idx = bct.block_count + i;
        let label = bct.graph.node_weight(NodeIndex::new(idx)).unwrap();
        cut_vertices_labels.insert(*label);
        output.push_str(&format!(
            "  cut{} [label=\"{}\", fillcolor=lightcoral];\n",
            label, label
        ));
    }

    // Draw edges between blocks (cloned cut vertices) and cut vertices
    for (i, block) in bct.blocks.iter().enumerate() {
        for node in block.node_indices() {
            let label = block.node_weight(node).unwrap();
            if cut_vertices_labels.contains(label) {
                // This is a cut vertex
                output.push_str(&format!(
                    "  b_{}_{} -- cut{} [style=dashed, penwidth=3];\n",
                    i,
                    label,
                    label
                ));
            }
        }
    }

    output.push_str("}\n");
    output
}

/// Draws the DFS tree and indicates cut vertices.
///
/// Tree edges are drawn in solid lines, back edges in dashed lines.
///
/// Cut vertices are colored red.
///
/// Intended to use with `dot`.
pub fn draw_bc_tree_dfs(
    graph: &UnGraph,
    bc_tree: &BlockCutTree,
) -> String {
    let mut output = String::from("digraph {\n");
    output.push_str("  rankdir=TD;\n");
    output.push_str("  node [style=filled, shape=circle];\n");

    for (i, node) in graph.node_indices().enumerate() {
        let label = graph.node_weight(node).unwrap();
        let color = if bc_tree.node_to_id[node.index()] < bc_tree.block_count {
            "lightblue"
        } else {
            "lightcoral"
        };
        output.push_str(&format!(
            "  {} [label=\"{}\", fillcolor={}];\n",
            i, label, color
        ));
    }

    // Add edges with labels
    for edge in graph.edge_references() {
        let (mut a, mut b) = (edge.source().index(), edge.target().index());
        let label = bc_tree.edge_labels[edge.id().index()].clone();
        let style = match label {
            DFSEdgeLabel::Tree => {
                if bc_tree.preorder[a] > bc_tree.preorder[b] {
                    std::mem::swap(&mut a, &mut b);
                }
                "solid"
            },
            DFSEdgeLabel::Back => {
                if bc_tree.preorder[a] < bc_tree.preorder[b] {
                    std::mem::swap(&mut a, &mut b);
                }
                "dashed"
            },
            _ => "",
        };
        output.push_str(&format!(
            "  {} -> {} [style={}];\n",
            a, b, style
        ));
    }

    output.push_str("}\n");
    output
}

#[cfg(test)]
mod dfs_tests {
    use super::*;
    use crate::types::UnGraph;

    fn run_dfs(g: &UnGraph, start: usize) -> (Vec<bool>, Vec<Vec<usize>>, Vec<usize>) {
        let mut time = 0;
        let mut preorder = vec![usize::MAX; g.node_count()];
        let mut edge_labels = vec![DFSEdgeLabel::Unvisited; g.edge_count()];
        let mut edge_stack = Vec::new();
        let mut blocks = Vec::new();
        let mut is_cut = vec![false; g.node_count()];
        dfs(
            g,
            start,
            None,
            &mut time,
            &mut preorder,
            &mut edge_labels,
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

    // In addition,
    // https://judge.yosupo.jp/submission/296498

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

#[cfg(test)]
mod bc_tests {
    use petgraph::visit::{IntoNodeReferences, NodeFiltered};
    use petgraph::graph::{DiGraph};
    use petgraph::algo::{ford_fulkerson};
    use petgraph::visit::{Dfs};

    use crate::testing::graph_enumerator::GraphEnumeratorState;
    use crate::testing::random_graphs::{random_connected_graph};
    use super::*;

    /// Based on Menger's theorem and flow networks.
    fn are_biconnected_flows(in_graph: &UnGraph) -> Vec<Vec<bool>> {
        let n = in_graph.node_references().count();
        if n == 1 {
            return vec![vec![false]];
        } else if n == 2 {
            return vec![vec![false, true], vec![true, false]];
        }

        let mut network = DiGraph::<usize, usize>::new();
        let mut res = vec![vec![false; n]; n];
        for i in 0..2 * n {
            network.add_node(i);
        }
        for (u, v) in in_graph
            .edge_references()
            .map(|e| (e.source().index(), e.target().index())) {
            network.add_edge(NodeIndex::new(u + n), NodeIndex::new(v), 1);
            network.add_edge(NodeIndex::new(v + n), NodeIndex::new(u), 1);
        }

        for i in 0..n {
            network.add_edge(NodeIndex::new(i), NodeIndex::new(i + n), 1);
        }

        for u in 0..n {
            for v in 0..n {
                if u == v {
                    continue;
                }
                res[u][v] = ford_fulkerson(&network, NodeIndex::new(u + n), NodeIndex::new(v)).0 > 1;
            }
        }

        res
    }

    fn check_if_component_is_biconnected(
        component: &UnGraph
    ) -> bool {
        if component.node_count() <= 2 {
            return true;
        }

        let matrix = are_biconnected_flows(component);
        let sum: usize = matrix.iter()
            .map(|row| row.iter().filter(|&&b| b).count())
            .sum();
        sum == component.node_count() * (component.node_count() - 1) // matrix[v,v] = false
    }

    fn find_articulation_points_brute(
        graph: &UnGraph,
    ) -> Vec<bool> {
        let n = graph.node_count();
        if n <= 2 {
            return vec![false; n]; // no articulation points in single node or two nodes
        }

        let mut is_cut = vec![false; n];
        for u in 0..n {
            let filtered_graph = NodeFiltered::from_fn(graph, |i| i.index() != u);
            let start = (0..n).find(|&v| v != u).unwrap();
            let mut dfs = Dfs::new(&filtered_graph, NodeIndex::new(start));
            let mut count = 0;
            while let Some(_) = dfs.next(&filtered_graph) {
                count += 1;
            }
            if count < n - 1 {
                is_cut[u] = true; // if we cannot reach all nodes, u is a cut vertex
            }
        }
        is_cut
    }

    fn is_connected(graph: &UnGraph) -> bool {
        let mut dfs = Dfs::new(graph, NodeIndex::new(0));
        let mut visited = 0;
        while let Some(_node) = dfs.next(graph) {
            visited += 1;
        }
        visited == graph.node_count()
    }

    fn glue_bc_tree_back(
        bct: &BlockCutTree,
    ) -> Vec<(usize, usize)> {
        let mut edges = vec![];

        for block in bct.blocks.iter() {
            for edge in block.edge_references() {
                let (u, v) = (edge.source(), edge.target());
                edges.push((*block.node_weight(u).unwrap() as usize, *block.node_weight(v).unwrap() as usize));
            }
        }

        edges
    }

    #[test]
    fn test_bc_tree_components_are_biconnected_light() {
        for i in 0..100 {
            let n = 2 + i / 10;
            let m: usize = 1 + i;

            let in_graph = random_connected_graph(n, m, i);

            let bct = get_block_cut_tree(&in_graph);
            for block in bct.blocks.iter() {
                assert!(check_if_component_is_biconnected(block));
            }
        }
    }

    fn get_cut_vertices_from_bct(bct: &BlockCutTree) -> Vec<bool> {
        let mut is_cut = vec![0; bct.node_to_id.len()]; // only cut vertices appear more than once
        for block in bct.blocks.iter() {
            for u in block.node_indices() {
                is_cut[*block.node_weight(u).unwrap() as usize] += 1;
            }
        }
        is_cut.into_iter().map(|x| x > 1).collect()
    }

    #[cfg(all(test, not(debug_assertions)))]
    #[test]
    fn test_bc_tree_components_are_biconnected_exhaustive() {
        // tests all connected simple graphs with n <= 7
        for n in 2..=7 {
            let mut enumerator = GraphEnumeratorState {
                n,
                mask: 0,
                last_mask: (1 << (n * (n - 1) / 2)),
            };

            while let Some(in_graph) = enumerator.next() {
                if !is_connected(&in_graph) {
                    continue; // not connected
                }

                let bct = get_block_cut_tree(&in_graph);

                for block in bct.blocks.iter() {
                    assert!(check_if_component_is_biconnected(block));
                }
            }
        }
    }

    #[test]
    fn test_bc_tree_cut_vertices_light() {
        for i in 0..100 {
            let n = 2 + i / 10;
            let m: usize = 1 + i;

            let in_graph = random_connected_graph(n, m, i);

            let bct = get_block_cut_tree(&in_graph);
            let is_cut = find_articulation_points_brute(&in_graph);

            assert_eq!(bct.cut_count, is_cut.iter().filter(|&&x| x).count());
            assert_eq!(get_cut_vertices_from_bct(&bct), is_cut);
        }
    }

    #[cfg(all(test, not(debug_assertions)))]
    #[test]
    fn test_bc_tree_cut_vertices_exhaustive() {
        // tests all connected simple graphs with n <= 7
        for n in 2..=7 {
            let mut enumerator = GraphEnumeratorState {
                n,
                mask: 0,
                last_mask: (1 << (n * (n - 1) / 2)),
            };

            while let Some(in_graph) = enumerator.next() {
                if !is_connected(&in_graph) {
                    continue; // not connected
                }

                let bct = get_block_cut_tree(&in_graph);
                let is_cut = find_articulation_points_brute(&in_graph);

                assert_eq!(bct.cut_count, is_cut.iter().filter(|&&x| x).count());
                assert_eq!(get_cut_vertices_from_bct(&bct), is_cut);
            }
        }
    }

    #[test]
    fn test_bc_tree_glue_back() {
        for i in 0..100 {
            let n = 2 + i / 10;
            let m: usize = 1 + i;

            let in_graph = random_connected_graph(n, m, i);

            let bct = get_block_cut_tree(&in_graph);
            let mut glued_edges = glue_bc_tree_back(&bct);

            let mut original_edges = in_graph
                .edge_references()
                .map(|e| (e.source().index(), e.target().index()))
                .collect::<Vec<_>>();

            glued_edges.sort();
            original_edges.sort();

            assert_eq!(glued_edges, original_edges);

        }
    }

    #[cfg(all(test, not(debug_assertions)))]
    #[test]
    fn test_bc_tree_glue_back_exhaustive() {
        // tests all connected simple graphs with n <= 7
        for n in 2..=7 {
            let mut enumerator = GraphEnumeratorState {
                n,
                mask: 0,
                last_mask: (1 << (n * (n - 1) / 2)),
            };

            while let Some(in_graph) = enumerator.next() {
                if !is_connected(&in_graph) {
                    continue; // not connected
                }

                let bct = get_block_cut_tree(&in_graph);
                let mut glued_edges = glue_bc_tree_back(&bct);

                let mut original_edges = in_graph
                    .edge_references()
                    .map(|e| (e.source().index(), e.target().index()))
                    .collect::<Vec<_>>();

                glued_edges.sort();
                original_edges.sort();

                assert_eq!(glued_edges, original_edges);

            }
        }
    }
}