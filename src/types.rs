/// Enum representing the type of edge in a graph.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EdgeLabel {
    Real,
    Virtual,
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

/// Wrapper for petgraph's graph type.
pub type UnGraph = petgraph::graph::UnGraph<u32, EdgeLabel>;
pub type DiGraph = petgraph::graph::DiGraph<u32, EdgeLabel>;

/// Enum to mark edges in DFS tree.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DFSEdgeLabel {
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
