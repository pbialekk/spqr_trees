#![warn(missing_docs)]

//! # spqr_trees
//!
//! A Rust library for learning what are SPQR trees,
//! how are they built and how can be used.
//! 
//! Based on [`petgraph`](https://docs.rs/petgraph).
//! 
//! TODO: add examples of usage later

pub mod types;
pub mod input;
pub mod output;
pub mod palm_tree;
pub mod parallel_edges;

pub use types::UnGraph;
pub use types::EdgeLabel;