//! Passes that move values or computations across region boundaries.

pub use self::{
	head_controlled_loop_inverter::HeadControlledLoopInverter,
	invariant_port_mover::InvariantPortMover,
	repeat_condition_canonicalizer::canonicalize_repeat_condition,
};

mod head_controlled_loop_inverter;
mod invariant_port_mover;
mod repeat_condition_canonicalizer;
