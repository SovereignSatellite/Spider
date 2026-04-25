//! Register coloring: tracks live registers and picks free slots by kind.

use alloc::vec::Vec;

use ir_graph::Link;

use crate::policy::Policy;

#[derive(Clone, Copy)]
struct Active {
	end: u32,
	register: u32,
	mask: u8,
}

#[derive(Clone, Copy)]
struct Slot {
	id: u32,
	mask: u8,
}

#[derive(Clone)]
pub struct Snapshot {
	active: Vec<Active>,
	free: Vec<Slot>,
	spill_cursor: u32,
}

pub struct Coloring {
	active: Vec<Active>,
	free: Vec<Slot>,
	spill_cursor: u32,
}

impl Coloring {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			active: Vec::new(),
			free: Vec::new(),
			spill_cursor: 0,
		}
	}

	pub fn reset(&mut self, policy: &dyn Policy) {
		self.active.clear();
		self.free.clear();
		self.free.extend(
			policy
				.registers()
				.iter()
				.zip(0_u32..)
				.map(|(&mask, id)| Slot { id, mask }),
		);
		self.free
			.sort_unstable_by_key(|slot| (slot.mask.count_ones(), slot.id));

		self.spill_cursor = u32::try_from(policy.registers().len()).unwrap();
	}

	fn insert_free(&mut self, slot: Slot) {
		let rank = (slot.mask.count_ones(), slot.id);
		let position = self
			.free
			.partition_point(|existing| (existing.mask.count_ones(), existing.id) < rank);

		self.free.insert(position, slot);
	}

	fn retire_where<P: Fn(u32) -> bool>(&mut self, start: usize, should_retire: P) {
		let mut index = start;

		while index < self.active.len() {
			if should_retire(self.active[index].end) {
				let retired = self.active.swap_remove(index);

				self.insert_free(Slot {
					id: retired.register,
					mask: retired.mask,
				});
			} else {
				index += 1;
			}
		}
	}

	#[must_use]
	pub const fn active_len(&self) -> usize {
		self.active.len()
	}

	pub fn retire_before(&mut self, start: usize, boundary: u32) {
		self.retire_where(start, |end| end < boundary);
	}

	pub fn retire_finished(&mut self, start: usize, id: u32) {
		self.retire_where(start, |end| end <= id);
	}

	fn take_matching(&mut self, kind: u8) -> Option<Slot> {
		let position = self.free.iter().position(|slot| slot.mask & kind == kind)?;

		Some(self.free.remove(position))
	}

	fn take(&mut self, kind: u8) -> (u32, u8) {
		if let Some(slot) = self.take_matching(kind) {
			return (slot.id, slot.mask);
		}

		let id = self.spill_cursor;

		self.spill_cursor += 1;

		(id, u8::MAX)
	}

	pub fn allocate(&mut self, policy: &dyn Policy, scope: usize, source: Link, end: u32) -> u32 {
		let kind = policy.kind(scope, source);
		let (register, mask) = self.take(kind);

		self.active.push(Active {
			end,
			register,
			mask,
		});

		register
	}

	#[must_use]
	pub fn snapshot(&self) -> Snapshot {
		Snapshot {
			active: self.active.clone(),
			free: self.free.clone(),
			spill_cursor: self.spill_cursor,
		}
	}

	pub fn restore(&mut self, snapshot: &Snapshot) {
		self.active.clear();
		self.active.extend_from_slice(&snapshot.active);
		self.free.clear();
		self.free.extend_from_slice(&snapshot.free);
		self.spill_cursor = snapshot.spill_cursor;
	}
}

impl Default for Coloring {
	fn default() -> Self {
		Self::new()
	}
}
