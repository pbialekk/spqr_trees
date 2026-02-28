//! Internal building blocks for the triconnected components algorithm.
//!
//! Contains the graph representation, DFS routines, path-finding, and
//! component merging logic used by [`crate::triconnected::get_triconnected_components`].

pub(crate) mod acceptable_adj;
pub(crate) mod graph_internal;
pub(crate) mod handle_duplicate_edges;
pub(crate) mod merge_components;
pub(crate) mod palm_dfs;
pub(crate) mod pathfinder;

pub mod outside_structures;
pub mod visualize;
