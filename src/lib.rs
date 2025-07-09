// #![warn(missing_docs)]

//! # spqr_trees
//!
//! A Rust library for learning what are SPQR trees,
//! how are they built and how can be used.
//!
//! Based on [`petgraph`](https://docs.rs/petgraph).
//!
//! TODO: add examples of usage later
//!
//! TODO: include papers

pub mod block_cut;
pub mod debugging;
pub mod input;
pub mod output;
pub mod palm_tree;
pub mod parallel_edges;
pub mod reduce;
pub mod triconnected;
pub mod tsin;
pub mod types;

pub use types::EdgeLabel;
pub use types::UnGraph;
