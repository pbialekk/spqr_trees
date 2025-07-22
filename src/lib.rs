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
pub mod input;
pub mod output;

pub mod triconnected;
pub mod triconnected_blocks;

pub mod spqr_blocks;
pub mod spqr_tree;
pub(crate) mod testing;

pub mod embedding;
pub(crate) mod embedding_blocks;

pub mod example_usages;

pub mod types;

pub use types::DFSEdgeLabel;
pub use types::EdgeLabel;
pub use types::UnGraph;
