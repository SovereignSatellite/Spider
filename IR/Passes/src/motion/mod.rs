//! Passes that move values or computations across region boundaries.

pub use self::invariant_port_mover::InvariantPortMover;

mod invariant_port_mover;
