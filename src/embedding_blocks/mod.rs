//! Internal building blocks for the LR planarity testing and embedding algorithm.
//!
//! Contains the graph representation, DFS orientation, conflict pair logic,
//! and embedding construction used by [`crate::embedding::is_planar`].

pub(crate) mod acceptable_adj;
pub(crate) mod embed;
pub(crate) mod kuratowski;
pub(crate) mod lr;
pub(crate) mod orient;
pub(crate) mod structures;
