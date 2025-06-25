/// Enum representing the type of edge in a graph.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EdgeLabel {
    Real,
    Virtual,
}

impl std::fmt::Display for EdgeLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeLabel::Real => write!(f, "Real"),
            EdgeLabel::Virtual => write!(f, "Virtual"),
        }
    }
}


/// Wrapper for petgraph's graph type.
pub type UnGraph = petgraph::graph::UnGraph<u32, EdgeLabel>;