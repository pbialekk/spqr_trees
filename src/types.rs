//! Core type definitions used throughout the library.

/// Enum representing the type of edge in a graph.
///
/// - `Real` — an edge from the original input graph.
/// - `Virtual` — a placeholder edge representing a split pair in the SPQR decomposition.
/// - `Structure` — an edge in a structural tree (e.g., block-cut tree).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EdgeLabel {
    /// An edge from the original input graph.
    Real,
    /// A placeholder edge representing a split pair.
    Virtual,
    /// An edge connecting structural nodes (e.g., in a block-cut tree).
    Structure,
}

impl std::fmt::Display for EdgeLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeLabel::Real => write!(f, "Real"),
            EdgeLabel::Virtual => write!(f, "Virtual"),
            EdgeLabel::Structure => write!(f, "Structure"),
        }
    }
}

/// Wrapper for petgraph's undirected graph type with `u32` node weights and [`EdgeLabel`] edge weights.
pub type UnGraph = petgraph::graph::UnGraph<u32, EdgeLabel>;
/// Wrapper for petgraph's directed graph type with `u32` node weights and [`EdgeLabel`] edge weights.
pub type DiGraph = petgraph::graph::DiGraph<u32, EdgeLabel>;

/// Enum to mark edges in a DFS tree.
///
/// - `Unvisited` — edge has not yet been classified.
/// - `Tree` — edge is part of the DFS spanning tree.
/// - `Back` — edge connects a descendant to an ancestor (back edge).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DFSEdgeLabel {
    /// Edge has not yet been classified.
    Unvisited,
    /// Edge is part of the DFS spanning tree.
    Tree,
    /// Back edge connecting a descendant to an ancestor.
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
