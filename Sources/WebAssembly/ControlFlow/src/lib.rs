//! Provide WebAssembly control-flow graphs and their structural analyses.

#![no_std]

extern crate alloc;

pub use self::{
	dot::Dot,
	graph::{BasicBlock, ControlFlowGraph},
	local_liveness::{LiveLocals, LocalLiveness},
	structure::ControlFlowStructurer,
	topological_order::TopologicalOrderer,
};

mod dot;
mod graph;
mod local_liveness;
mod structure;
mod topological_order;

pub mod instruction;
