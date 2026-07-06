//! Pre-coloring copy coalescing: same-value unification, then aggressive
//! heaviest-first class merging gated on footprint disjointness.
//!
//! Interference is value-based (Boissinot et al.): a value fed by exactly one
//! source value holds that value, so the two share a class even though their
//! intervals overlap. Coalescing runs before any register is chosen (Bouchez,
//! Darte, Rastello), so merging is never blocked by register scarcity.

use alloc::vec::Vec;
use core::cmp::Reverse;

use crate::{classes::Classes, policy::Policy, value};

const UNSET: u32 = u32::MAX - 1;
const CONFLICTED: u32 = u32::MAX;

// Spill slots, at or beyond the physical file, accept every kind.
fn accepts(registers: &[u8], register: u32, kind: u8) -> bool {
	registers
		.get(usize::try_from(register).unwrap())
		.is_none_or(|&mask| mask & kind == kind)
}

/// Coalesces copy edges into the class table before any register is chosen.
pub struct Coalescer {
	edges: Vec<(u32, u32)>,
	weighted: Vec<(u32, u32, u32)>,
}

impl Coalescer {
	/// Creates a new coalescer.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			edges: Vec::new(),
			weighted: Vec::new(),
		}
	}

	/// Folds every coalescable copy edge into the class table.
	pub fn run(&mut self, policy: &dyn Policy, classes: &mut Classes, copies: &[(u32, u32)]) {
		self.unify(classes, copies);
		self.weigh(classes, copies);
		self.merge(policy, classes);
	}

	// A destination fed by one distinct class is a copy of that value and joins
	// it even where their intervals overlap. Same-class sources are self-moves,
	// not conflicts, and rotations feed carries from later values, so the pass
	// repeats until no class folds.
	fn unify(&mut self, classes: &mut Classes, copies: &[(u32, u32)]) {
		self.edges.clear();
		self.edges
			.extend(copies.iter().map(|&(from, to)| (to, from)));
		self.edges.sort_unstable();

		while self.unify_pass(classes) {}
	}

	fn unify_pass(&self, classes: &mut Classes) -> bool {
		let mut changed = false;
		let mut cursor = 0;

		while cursor < self.edges.len() {
			let destination = self.edges[cursor].0;
			let copy = classes.find(destination);
			let mut only = UNSET;

			while cursor < self.edges.len() && self.edges[cursor].0 == destination {
				let source = classes.find(self.edges[cursor].1);

				cursor += 1;

				if source == copy {
					continue;
				}

				if only == UNSET {
					only = source;
				} else if only != source {
					only = CONFLICTED;
				}
			}

			if only != UNSET && only != CONFLICTED {
				classes.union(only, copy);
				changed = true;
			}
		}

		changed
	}

	fn weigh(&mut self, classes: &mut Classes, copies: &[(u32, u32)]) {
		self.edges.clear();

		for &(from, to) in copies {
			let first = classes.find(from);
			let second = classes.find(to);

			if first != second {
				self.edges.push((first.min(second), first.max(second)));
			}
		}

		self.edges.sort_unstable();

		self.weighted.clear();

		for &(left, right) in &self.edges {
			match self.weighted.last_mut() {
				Some(last) if last.0 == left && last.1 == right => last.2 += 1,
				_ => self.weighted.push((left, right, 1)),
			}
		}

		self.weighted.sort_by_key(|entry| Reverse(entry.2));
	}

	// Endpoints re-resolve at attempt time because classes merge as the pass
	// runs. A blocked edge never becomes mergeable later, since footprints,
	// pins, and kind unions only grow, so one pass is complete.
	fn merge(&self, policy: &dyn Policy, classes: &mut Classes) {
		for &(first, second, _) in &self.weighted {
			let left = classes.find(first);
			let right = classes.find(second);

			if left == right {
				continue;
			}

			let left_pin = classes.pin(left);
			let right_pin = classes.pin(right);

			if left_pin != value::NONE && right_pin != value::NONE && left_pin != right_pin {
				continue;
			}

			// The pinned side absorbs, so a parameter class stays rooted at its
			// own value id, which the sweeper pre-activates by index.
			let (into, from) = if right_pin == value::NONE {
				(left, right)
			} else {
				(right, left)
			};
			let pin = classes.pin(into);
			let kind = classes.kind(left) | classes.kind(right);
			let is_placeable = pin == value::NONE || accepts(policy.registers(), pin, kind);

			if is_placeable && classes.is_disjoint(left, right) {
				classes.union(into, from);
			}
		}
	}
}
