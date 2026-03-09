//! Visitor-based transformations and analysis passes for the IR graph.

#![no_std]

extern crate alloc;

pub mod control;
pub mod isle;
pub mod successor_finder;
pub mod topological_normalizer;
