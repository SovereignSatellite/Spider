//! Region-local transformation and analysis passes for the IR graph.

extern crate alloc;

pub mod analysis;
pub mod catalog;
pub mod motion;
pub mod normalize;
pub mod simplify;
