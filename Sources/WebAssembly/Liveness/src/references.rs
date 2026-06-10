//! External reference tracking.

use alloc::vec::Vec;

use web_assembly_graph::instruction::{Instruction, Reference};

/// Collects all external references used by the given instructions.
pub fn track(references: &mut Vec<Reference>, instructions: &[Instruction]) {
	references.clear();

	for &instruction in instructions {
		instruction.for_each_reference(|reference| references.push(reference));
	}

	references.sort_unstable();
	references.dedup();
}
