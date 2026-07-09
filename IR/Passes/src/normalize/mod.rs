//! Shape passes that establish the backend contract for every region.

pub use self::{
	dead_port_eliminator::DeadPortEliminator, topological_compactor::TopologicalCompactor,
};

mod dead_port_eliminator;
mod topological_compactor;

pub mod identity;
