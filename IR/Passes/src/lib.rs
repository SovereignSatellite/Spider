//! Region-local transformation and analysis passes for the IR graph.

extern crate alloc;

mod tracer;

pub mod dead_port_eliminator;
pub mod identity;
pub mod invariant_port_mover;
pub mod isle;
pub mod successor_finder;
pub mod topological_compactor;
