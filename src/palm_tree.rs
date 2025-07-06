use crate::UnGraph;
use fixedbitset::FixedBitSet;
use hashbrown::HashMap;
use petgraph::visit::{EdgeRef, IntoNodeReferences, NodeIndexable, NodeRef};
use std::usize;

/// Computes low points/palm tree of a graph.
///
/// TODO: explain what low points are and rewrite ifs in dfs, change parent type to Optional.
///
/// Graph should be undirected, connected and simple.
///
/// See `mod parallel_edges`
pub fn get_palm_tree(g: &UnGraph) -> PalmTree {
    let graph_size = g.node_references().size_hint().0;
    let edges_size = g.edge_references().size_hint().0;
    let mut palm_tree = PalmTree::new(graph_size, edges_size);

    let root = g
        .node_references()
        .next()
        .expect("Graph should not be empty");
    let root_id = g.to_index(root.id());
    dfs(&g, root_id, usize::MAX, &mut palm_tree);

    palm_tree
}

/// Returns a string representation of the palm tree in dot format.
///
/// LOWS are ids that you gave to nodes in the graph. They are not discovery times.
///
/// Tree edges are solid, back edges are dotted.
///
/// The root node is colored green.
///
/// Use returned string with `dot` not `neato`.
pub fn draw_palm_tree(palm_tree: &PalmTree, g: &UnGraph) -> String {
    let mut dot_str = String::new();
    dot_str.push_str("digraph {\n");
    dot_str.push_str("  node [style=filled, shape=ellipse];\n");

    for node in g.node_references() {
        let node_id = g.to_index(node.id());
        let node_label = node.weight();
        let low1 = palm_tree.rank_to_node[&palm_tree.low1[node_id]];
        let low2 = palm_tree.rank_to_node[&palm_tree.low2[node_id]];
        let low1_label = g.node_weight(g.from_index(low1)).unwrap();
        let low2_label = g.node_weight(g.from_index(low2)).unwrap();
        let dfs_time = palm_tree.rank[node_id];

        // Coloring: root is green, others are lightblue
        let color = if palm_tree.parent[node_id] == usize::MAX {
            "green"
        } else {
            "lightblue"
        };

        dot_str.push_str(&format!(
            "  {} [label=\"ID:{} LOWS: {} | {}\nRANK: {}\", fillcolor={}];\n",
            node_id, node_label, low1_label, low2_label, dfs_time, color
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
            DFSEdgeLabel::Tree => {
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
            DFSEdgeLabel::Back => "style=\"dotted\"",
            _ => "",
        };

        dot_str.push_str(&format!(
            "  {} -> {} [{}];\n",
            from, to, style
        ));
    }

    dot_str.push_str("}\n");
    dot_str
}

/// Enum to mark edges in DFS tree.
#[derive(Clone, PartialEq, Eq, Debug)]
enum DFSEdgeLabel {
    Unvisited,
    Tree,
    Back,
}

impl std::fmt::Display for DFSEdgeLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DFSEdgeLabel::Unvisited => write!(f, "Unvisited"),
            DFSEdgeLabel::Tree => write!(f, "Tree"),
            DFSEdgeLabel::Back => write!(f, "Back"),
        }
    }
}

/// This struct holds all information about the palm tree of the graph.
#[derive(Debug)]
pub struct PalmTree {
    // usize is there to keep petgraph's convention
    visited: FixedBitSet,
    low1: Vec<usize>,
    low2: Vec<usize>,
    rank: Vec<usize>,
    rank_to_node: HashMap<usize, usize>,
    descendants: Vec<usize>,
    parent: Vec<usize>,
    time: usize,
    edge_labels: Vec<DFSEdgeLabel>,
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
            edge_labels: vec![DFSEdgeLabel::Unvisited; edges_size],
        }
    }
}

/// Helper that performs the required DFS in a recursive manner.
fn dfs(g: &UnGraph, current_node: usize, _: usize, palm_tree: &mut PalmTree) {
    palm_tree.rank[current_node] = palm_tree.time;
    palm_tree.rank_to_node.insert(palm_tree.time, current_node);
    palm_tree.visited.insert(current_node);
    palm_tree.low1[current_node] = palm_tree.rank[current_node];
    palm_tree.low2[current_node] = palm_tree.rank[current_node];
    palm_tree.descendants[current_node] = 1;
    palm_tree.time += 1;

    for edge in g.edges(g.from_index(current_node)) {
        let edge_index = edge.id().index();
        if palm_tree.edge_labels[edge_index] != DFSEdgeLabel::Unvisited {
            continue;
        }

        let child_node = g.to_index(edge.target());
        if !palm_tree.visited.contains(child_node) {
            let edge_index = edge.id().index();
            palm_tree.edge_labels[edge_index] = DFSEdgeLabel::Tree;
            palm_tree.parent[child_node] = current_node;

            dfs(g, child_node, current_node, palm_tree);

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
            palm_tree.edge_labels[edge_index] = DFSEdgeLabel::Back;
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

// TODO: tests
