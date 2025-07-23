use crate::triconnected_blocks::graph_internal::GraphInternal;

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    Tree,
    Back,
    Killed,
}

/// Represents the type of a component in the triconnected block decomposition.
///
/// - `P`: Bond (parallel edges, k >= 3)
/// - `S`: Cycle (simple cycle)
/// - `R`: Triconnected component (rigid)
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ComponentType {
    P,      // bond
    S,      // triangle
    R,      // triconnected
    UNSURE, // used for initial state
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentType::P => write!(f, "P"),
            ComponentType::S => write!(f, "S"),
            ComponentType::R => write!(f, "R"),
            &ComponentType::UNSURE => {
                panic!();
            }
        }
    }
}

/// Represents a component in the triconnected block decomposition.
///
/// Contains a list of edges that belong to the component and its type.
#[derive(Debug, Clone)]
pub struct Component {
    pub edges: Vec<usize>,
    pub comp_type: ComponentType,
}

impl Component {
    pub fn new(comp_type: ComponentType) -> Self {
        Self {
            edges: Vec::new(),
            comp_type: comp_type,
        }
    }

    pub fn push_edge(
        &mut self,
        eid: usize,
        graph: &mut GraphInternal,
        is_virtual: bool,
    ) -> &mut Self {
        self.edges.push(eid);
        if !is_virtual {
            graph.remove_edge(eid);
        }

        self
    }

    pub fn commit(&mut self, split_components: &mut Vec<Component>) {
        if self.comp_type == ComponentType::UNSURE {
            self.comp_type = if self.edges.len() >= 4 {
                ComponentType::R
            } else {
                ComponentType::S
            };
        }

        split_components.push(self.clone());
    }
}

/// Holds the triconnected components of a graph.
///
/// Contains a list of components, edges, and additional metadata about the edges.
///
/// - `comp`: List of components in the triconnected decomposition.
/// - `edges`: List of edges in the graph. Also contains the virtual edges created during the splitting process.
/// - `is_real`: Indicates if an edge is a real edge in the original graph.
/// - `to_split`: Maps edges to their corresponding split components. Virtual edges are mapped to `None`.
#[derive(Debug, Clone)]
pub struct TriconnectedComponents {
    pub comp: Vec<Component>,
    pub edges: Vec<(usize, usize)>,
    pub is_real: Vec<bool>,
    pub to_split: Vec<Option<usize>>,
}
