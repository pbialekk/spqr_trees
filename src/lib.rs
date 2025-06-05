//! # spqr_trees
//!
//! A Rust library for learning what are SPQR trees,
//! how are they built and how can be used.  
//! Based on [`petgraph`](https://docs.rs/petgraph).

pub mod input;
pub mod output;
pub mod palm_tree;
pub mod parallel_edges;

pub use input::from_file;
pub use input::from_str;
pub use output::to_dot_file;
pub use output::to_dot_str;
pub use palm_tree::get_palm_tree;
pub use palm_tree::draw_palm_tree;