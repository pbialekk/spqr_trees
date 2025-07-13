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
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    P, // bond
    S, // triangle
    R, // triconnected
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentType::P => write!(f, "P"),
            ComponentType::S => write!(f, "S"),
            ComponentType::R => write!(f, "R"),
        }
    }
}

/// Represents a component in the triconnected block decomposition.
///
/// Contains a list of edges that belong to the component and its type.
#[derive(Debug, Clone)]
pub struct Component {
    pub edges: Vec<usize>,
    pub component_type: Option<ComponentType>,
}

impl Component {
    pub(crate) fn new(component_type: Option<ComponentType>) -> Self {
        Self {
            edges: Vec::new(),
            component_type,
        }
    }

    pub(crate) fn push_edge(
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

    pub(crate) fn commit(&mut self, split_components: &mut Vec<Component>) {
        if self.component_type.is_none() {
            self.component_type = Some(if self.edges.len() >= 4 {
                ComponentType::R
            } else {
                ComponentType::S
            });
        }

        split_components.push(self.clone());
    }
}

/// Holds the triconnected components of a graph.
///
/// Contains a list of components, edges, and additional metadata about the edges.
///
/// - `components`: List of components in the triconnected decomposition.
/// - `edges`: List of edges in the original graph.
/// - `is_real_edge`: Indicates if an edge is a real edge in the original graph.
/// - `real_to_split`: Maps real edges to their corresponding split components, if any.
#[derive(Debug, Clone)]
pub struct TriconnectedComponents {
    pub components: Vec<Component>,
    pub edges: Vec<(usize, usize)>,
    pub is_real_edge: Vec<bool>,
    pub real_to_split: Vec<Option<usize>>,
}
