use fixedbitset::FixedBitSet;
use hashbrown::HashMap;
use petgraph::graph::UnGraph;
use petgraph::visit::{EdgeRef, IntoNodeReferences, NodeIndexable, NodeRef};
use std::usize;

/// Computes low points/palm tree of a graph.
///
/// Graph should be undirected, connected and simple.
///
/// See `mod parallel_edges`
pub fn get_palm_tree(g: &UnGraph<u32, String>) -> PalmTree {
    let graph_size = g.node_references().size_hint().0;
    let edges_size = g.edge_references().size_hint().0;
    let mut palm_tree = PalmTree::new(graph_size, edges_size);

    let root = g
        .node_references()
        .next()
        .expect("Graph should not be empty");
    let root_id = g.to_index(root.id());
    _dfs(&g, root_id, usize::MAX, &mut palm_tree);

    palm_tree
}

/// Returns a string representation of the palm tree in dot format.
pub fn draw_palm_tree(palm_tree: &PalmTree, g: &UnGraph<u32, String>) -> String {
    let mut dot_str = String::new();
    dot_str.push_str("digraph PalmTree {\n");

    for node in g.node_references() {
        let node_id = g.to_index(node.id());
        let low1 = palm_tree
            .rank_to_node
            .get(&palm_tree.low1[node_id])
            .unwrap()
            .clone();
        let low2 = palm_tree
            .rank_to_node
            .get(&palm_tree.low2[node_id])
            .unwrap()
            .clone();

        // Example coloring: root is green, others are lightblue
        let color = if palm_tree.parent[node_id] == usize::MAX {
            "green"
        } else {
            "lightblue"
        };

        dot_str.push_str(&format!(
            "  {} [label=\"ID:{} LOWS: {}|{}\", style=filled, fillcolor={}];\n",
            node_id, node_id, low1, low2, color
        ));
    }

    for edge in g.edge_references() {
        let edge_index = edge.id().index();
        let source_id = g.to_index(edge.source());
        let target_id = g.to_index(edge.target());
        let label = &palm_tree.edge_labels[edge_index];

        let source_rank = palm_tree.rank[source_id];
        let target_rank = palm_tree.rank[target_id];

        let (from, to) = match label {
            EdgeLabel::Tree => {
                if source_rank < target_rank {
                    (source_id, target_id)
                } else {
                    (target_id, source_id)
                }
            }
            _ => {
                if source_rank > target_rank {
                    (source_id, target_id)
                } else {
                    (target_id, source_id)
                }
            }
        };

        let style = match label {
            EdgeLabel::Back => ", style=\"dotted\"",
            _ => "",
        };

        dot_str.push_str(&format!(
            "  {} -> {} [label=\"{}\"{}];\n",
            from, to, label, style
        ));
    }

    dot_str.push_str("}\n");
    dot_str
}

/// Enum to mark edges in DFS tree.
#[derive(Clone, PartialEq, Eq, Debug)]
enum EdgeLabel {
    Unvisited,
    Tree,
    Back,
}

impl std::fmt::Display for EdgeLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeLabel::Unvisited => write!(f, "Unvisited"),
            EdgeLabel::Tree => write!(f, "Tree"),
            EdgeLabel::Back => write!(f, "Back"),
        }
    }
}

/// This struct holds all information about the palm tree of the graph.
#[derive(Debug)]
pub struct PalmTree {
    visited: FixedBitSet,
    low1: Vec<usize>,
    low2: Vec<usize>,
    rank: Vec<usize>,
    rank_to_node: HashMap<usize, usize>,
    descendants: Vec<usize>,
    parent: Vec<usize>,
    time: usize,
    edge_labels: Vec<EdgeLabel>,
}

impl PalmTree {
    fn new(graph_size: usize, edges_size: usize) -> Self {
        Self {
            visited: FixedBitSet::with_capacity(graph_size),
            low1: vec![usize::MAX; graph_size],
            low2: vec![usize::MAX; graph_size],
            rank: vec![usize::MAX; graph_size],
            rank_to_node: HashMap::with_capacity(graph_size),
            descendants: vec![usize::MAX; graph_size],
            parent: vec![usize::MAX; graph_size],
            time: 0,
            edge_labels: vec![EdgeLabel::Unvisited; edges_size],
        }
    }
}

/// Helper that performs the required DFS in a recursive manner.
fn _dfs(g: &UnGraph<u32, String>, current_node: usize, _: usize, palm_tree: &mut PalmTree) {
    palm_tree.rank[current_node] = palm_tree.time;
    palm_tree.rank_to_node.insert(palm_tree.time, current_node);
    palm_tree.visited.insert(current_node);
    palm_tree.low1[current_node] = palm_tree.rank[current_node];
    palm_tree.low2[current_node] = palm_tree.rank[current_node];
    palm_tree.descendants[current_node] = 1;
    palm_tree.time += 1;

    for edge in g.edges(g.from_index(current_node)) {
        let edge_index = edge.id().index();
        if palm_tree.edge_labels[edge_index] != EdgeLabel::Unvisited {
            continue;
        }

        let child_node = g.to_index(edge.target());
        if !palm_tree.visited.contains(child_node) {
            let edge_index = edge.id().index();
            palm_tree.edge_labels[edge_index] = EdgeLabel::Tree;
            palm_tree.parent[child_node] = current_node;

            _dfs(g, child_node, current_node, palm_tree);

            if palm_tree.low1[child_node] < palm_tree.low1[current_node] {
                palm_tree.low2[current_node] =
                    std::cmp::min(palm_tree.low1[current_node], palm_tree.low2[child_node]);
                palm_tree.low1[current_node] = palm_tree.low1[child_node];
            } else if palm_tree.low1[child_node] == palm_tree.low1[current_node] {
                palm_tree.low2[current_node] =
                    std::cmp::min(palm_tree.low2[current_node], palm_tree.low2[child_node]);
            } else {
                palm_tree.low2[current_node] =
                    std::cmp::min(palm_tree.low2[current_node], palm_tree.low1[child_node]);
            }
            palm_tree.descendants[current_node] += palm_tree.descendants[child_node];
        } else {
            let edge_index = edge.id().index();
            palm_tree.edge_labels[edge_index] = EdgeLabel::Back;
            if palm_tree.rank[child_node] < palm_tree.low1[current_node] {
                palm_tree.low2[current_node] = palm_tree.low1[current_node];
                palm_tree.low1[current_node] = palm_tree.rank[child_node];
            } else if palm_tree.rank[child_node] > palm_tree.low1[current_node] {
                palm_tree.low2[current_node] =
                    std::cmp::min(palm_tree.low2[current_node], palm_tree.rank[child_node]);
            }
        }
    }
}

// /// Enum to represent state in recursion.
// enum RecursionStep {
//     BaseStep(usize),
//     UpdateLowsAndDesc(usize, usize),
// }

// /// Helper that performs the required DFS in an iterative manner.
// /// TODO: Incomplete for now
// fn _dfs_iterative(g: &UnGraph<u32, String>, target_node: usize, palm_tree: &mut PalmTree) {
//     let mut stack: Vec<RecursionStep> = vec![RecursionStep::BaseStep(target_node)];

//     while let Some(recursion_step) = stack.pop() {
//         match recursion_step {
//             RecursionStep::BaseStep(current_node) => {
//                 palm_tree.time += 1;
//                 palm_tree.rank[current_node] = palm_tree.time;
//                 palm_tree.visited.insert(current_node);
//                 palm_tree.low1[current_node] = palm_tree.rank[current_node];
//                 palm_tree.low2[current_node] = palm_tree.rank[current_node];
//                 palm_tree.descendants[current_node] = 1;

//                 for edge in g.edges(g.from_index(current_node)) {
//                     let child_node = g.to_index(edge.target());
//                     if !palm_tree.visited.contains(child_node) {
//                         let edge_index = edge.id().index();
//                         palm_tree.edge_labels[edge_index] = EdgeLabel::Tree;
//                         palm_tree.parent[child_node] = current_node;
//                         stack.push(RecursionStep::UpdateLowsAndDesc(current_node, child_node));
//                         stack.push(RecursionStep::BaseStep(child_node));
//                     } else {
//                         let edge_index = edge.id().index();
//                         palm_tree.edge_labels[edge_index] = EdgeLabel::Back;
//                     }
//                 }
//             }
//             RecursionStep::UpdateLowsAndDesc(current_node, child_node) => {
//                 if palm_tree.low1[child_node] < palm_tree.low1[current_node] {
//                     palm_tree.low2[current_node] =
//                         std::cmp::min(palm_tree.low1[current_node], palm_tree.low2[child_node]);
//                     palm_tree.low2[current_node] = palm_tree.low1[child_node];
//                 } else if palm_tree.low1[child_node] == palm_tree.low1[current_node] {
//                     palm_tree.low2[current_node] =
//                         std::cmp::min(palm_tree.low2[current_node], palm_tree.low2[child_node]);
//                 } else {
//                     palm_tree.low2[current_node] =
//                         std::cmp::min(palm_tree.low2[current_node], palm_tree.low1[child_node]);
//                 }
//                 palm_tree.descendants[current_node] += palm_tree.descendants[child_node];
//             }
//         }
//     }
// }
