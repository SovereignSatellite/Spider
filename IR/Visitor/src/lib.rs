//! Visitor-based transformations and analysis passes for the IR graph.

extern crate alloc;

pub mod control;
pub mod isle;
pub mod region_driver;
pub mod successor_finder;
pub mod topological_compactor;
