use crate::triconnected_blocks::graph_internal::GraphInternal;

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    Tree,
    Back,
    Killed,
}

/// Labels the types of components in the triconnected block decomposition.
/// - `P` for (k >= 3) bonds
/// - `S` for cycles,
/// - `R` for triconnected components.
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    P, // bond
    S, // triangle
    R, // triconnected
}

/// Represents a component in the triconnected block decomposition.
///
/// It contains a list of edges that belong to the corresponding component and its type.
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
