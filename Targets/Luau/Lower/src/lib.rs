//! Luau-specific lowering of trivial IR operations.
//!
//! Runs as the injected pass inside the optimizer fixpoint, expanding trivial operations
//! in place so the generic passes can compact and simplify the result.

use ir_graph::Region;

/// Lowers trivial operations in the region, reporting whether it changed anything.
#[must_use]
pub const fn apply(_region: &mut Region) -> bool {
	false
}
