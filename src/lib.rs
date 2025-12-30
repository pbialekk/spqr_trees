//! # spqr_trees
//!
//! A Rust library for learning what are SPQR trees,
//! how are they built and how can be used.
//!
//! Based on [petgraph's](https://docs.rs/petgraph) graph implementation.
//!
//! SPQR tree is a data structure that represents the decomposition of a graph into its `triconnected components`.
//!
//! Nodes are classified into three types:
//! - **S-nodes** - simple cycles
//! - **P-nodes** - bonds
//! - **R-nodes** - triconnected components
//!
//! Edges in nodes can be `real` or `virtual`.
//! `Virtual` edges represent `split pair` and some subgraph.
//!
//! Our implementation searches for `split pairs`.
//! Splits graph and then merges the components back together to form one of the above.
//!
//! SPQR trees are useful for various applications in graph theory, such network reliability - `triconnectivity queries` or
//! graph drawing - `embeddings` and `OEIP`.
//!
//! Our library provides many points where you can draw graphs which algorithms construct.
//!
//! See `output`. To visualize use local `dot` installation or [online tools](https://dreampuf.github.io/GraphvizOnline/?engine=dot#digraph%20G%20%7B%0A%0A%20%20subgraph%20cluster_0%20%7B%0A%20%20%20%20style%3Dfilled%3B%0A%20%20%20%20color%3Dlightgrey%3B%0A%20%20%20%20node%20%5Bstyle%3Dfilled%2Ccolor%3Dwhite%5D%3B%0A%20%20%20%20a0%20-%3E%20a1%20-%3E%20a2%20-%3E%20a3%3B%0A%20%20%20%20label%20%3D%20%22process%20%231%22%3B%0A%20%20%7D%0A%0A%20%20subgraph%20cluster_1%20%7B%0A%20%20%20%20node%20%5Bstyle%3Dfilled%5D%3B%0A%20%20%20%20b0%20-%3E%20b1%20-%3E%20b2%20-%3E%20b3%3B%0A%20%20%20%20label%20%3D%20%22process%20%232%22%3B%0A%20%20%20%20color%3Dblue%0A%20%20%7D%0A%20%20start%20-%3E%20a0%3B%0A%20%20start%20-%3E%20b0%3B%0A%20%20a1%20-%3E%20b3%3B%0A%20%20b2%20-%3E%20a3%3B%0A%20%20a3%20-%3E%20a0%3B%0A%20%20a3%20-%3E%20end%3B%0A%20%20b3%20-%3E%20end%3B%0A%0A%20%20start%20%5Bshape%3DMdiamond%5D%3B%0A%20%20end%20%5Bshape%3DMsquare%5D%3B%0A%7D)
//!
//! In addition, we provide implementation of `block-cut trees`. They store biconnected components of a graph.
//! You can use them to divide graph into biconnected subgraphs.
//!
//! For examples of usage, see `examples`, `src/example_usages` and `tests`.
pub mod block_cut;
pub mod input;
pub mod output;
pub mod spqr_blocks;
pub mod spqr_tree;
pub(crate) mod testing;
pub mod triconnected;
pub mod triconnected_blocks;

pub mod embedding;
pub(crate) mod embedding_blocks;

pub mod drawing_blocks;
pub mod example_usages;
pub mod types;

pub use types::DFSEdgeLabel;
pub use types::EdgeLabel;
pub use types::UnGraph;
