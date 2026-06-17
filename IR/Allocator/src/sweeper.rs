//! The interval sweeper: a single forward scan assigning registers to
//! start-sorted values.

use alloc::vec::Vec;

use crate::{
	policy::Policy,
	value::{self, Value},
};

#[derive(Clone, Copy)]
struct Slot {
	id: u32,
	mask: u8,
	is_reserved: bool,
}

impl Slot {
	// Free registers are handed out by this rank: unreserved before reserved, then
	// most-constrained kind first, then lowest id.
	const fn rank(self) -> (bool, u32, u32) {
		(self.is_reserved, self.mask.count_ones(), self.id)
	}

	const fn accepts(self, kind: u8) -> bool {
		self.mask & kind == kind
	}
}

#[derive(Clone, Copy)]
struct Active {
	end: u32,
	register: u32,
	is_reserved: bool,
	mask: u8,
}

impl Active {
	const fn new(value: &Value, register: u32, mask: u8) -> Self {
		Self {
			end: value.end,
			register,
			is_reserved: value.is_reserved,
			mask,
		}
	}

	const fn freed_slot(self) -> Slot {
		Slot {
			id: self.register,
			mask: self.mask,
			is_reserved: self.is_reserved,
		}
	}
}

/// Assigns registers to values by a single forward interval scan.
pub struct Sweeper {
	free: Vec<Slot>,
	active: Vec<Active>,
	order: Vec<u32>,
	assignments: Vec<u32>,
	physical_count: u32,
	spill_cursor: u32,
}

impl Sweeper {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			free: Vec::new(),
			active: Vec::new(),
			order: Vec::new(),
			assignments: Vec::new(),
			physical_count: 0,
			spill_cursor: 0,
		}
	}

	fn reset(&mut self, policy: &dyn Policy, parameter_count: u16) {
		self.active.clear();
		self.free.clear();
		self.free.extend(
			policy
				.registers()
				.iter()
				.zip(0_u32..)
				.skip(usize::from(parameter_count))
				.map(|(&mask, id)| Slot {
					id,
					mask,
					is_reserved: false,
				}),
		);
		self.free.sort_unstable_by_key(|slot| slot.rank());

		self.physical_count = u32::try_from(policy.registers().len()).unwrap();
		self.spill_cursor = self.physical_count.max(u32::from(parameter_count));
	}

	fn insert_free(&mut self, slot: Slot) {
		let slot_rank = slot.rank();
		let position = self
			.free
			.partition_point(|existing| existing.rank() < slot_rank);

		self.free.insert(position, slot);
	}

	fn expire(&mut self, time: u32) {
		let mut index = 0;

		while index < self.active.len() {
			if self.active[index].end < time {
				let retired = self.active.swap_remove(index);

				// A register a later value wants to coalesce onto is ranked behind the
				// rest, so first-fit hands out other registers first while the
				// coalescing partner can still reclaim this one by id.
				self.insert_free(retired.freed_slot());
			} else {
				index += 1;
			}
		}
	}

	fn has_physical_fit(&self, kind: u8) -> bool {
		self.free
			.iter()
			.any(|slot| slot.id < self.physical_count && slot.accepts(kind))
	}

	// Prefer the partner's register so the transfer coalesces away. A physical partner
	// is honored outright; a spilled partner only once no physical register fits, so
	// coalescing never manufactures a spill.
	fn partner_position(&self, kind: u8, preferred: u32) -> Option<usize> {
		let position = self
			.free
			.iter()
			.position(|slot| slot.id == preferred && slot.accepts(kind))?;
		let is_honored = preferred < self.physical_count || !self.has_physical_fit(kind);

		is_honored.then_some(position)
	}

	fn first_fit_position(&self, kind: u8) -> Option<usize> {
		self.free.iter().position(|slot| slot.accepts(kind))
	}

	fn take(&mut self, kind: u8, preferred: u32) -> (u32, u8) {
		let position = self
			.partner_position(kind, preferred)
			.or_else(|| self.first_fit_position(kind));

		if let Some(position) = position {
			let slot = self.free.remove(position);

			return (slot.id, slot.mask);
		}

		let id = self.spill_cursor;

		self.spill_cursor += 1;

		(id, u8::MAX)
	}

	// The collector mints the function argument ports as values 0..`parameter_count`
	// in port order, so pinning value `index` to register `index` keeps each
	// parameter in the register the calling convention already placed it in. A
	// parameter beyond the physical register file lands on a spill slot, which
	// accepts every kind.
	fn assign_parameters(&mut self, policy: &dyn Policy, parameter_count: u16, values: &[Value]) {
		for (index, value) in values.iter().enumerate().take(usize::from(parameter_count)) {
			let register = u32::try_from(index).unwrap();
			let mask = policy.registers().get(index).copied().unwrap_or(u8::MAX);

			self.assignments[index] = register;
			self.active.push(Active::new(value, register, mask));
		}
	}

	fn collect_order(&mut self, parameter_count: u16, values: &[Value]) {
		self.order.clear();
		self.order
			.extend(u32::from(parameter_count)..u32::try_from(values.len()).unwrap());
		self.order
			.sort_unstable_by_key(|&id| (values[usize::try_from(id).unwrap()].start, id));
	}

	fn preferred(&self, value: Value) -> u32 {
		if value.affinity == value::NONE {
			value::NONE
		} else {
			self.assignments[usize::try_from(value.affinity).unwrap()]
		}
	}

	fn scan(&mut self, values: &[Value]) {
		for index in 0..self.order.len() {
			let id = usize::try_from(self.order[index]).unwrap();
			let value = values[id];

			debug_assert!(value.start <= value.end, "intervals must be ordered");

			self.expire(value.start);

			let (register, mask) = self.take(value.kind, self.preferred(value));

			self.assignments[id] = register;
			self.active.push(Active::new(&value, register, mask));
		}
	}

	/// Assigns registers and returns the per-value assignment table.
	pub fn run(&mut self, policy: &dyn Policy, parameter_count: u16, values: &[Value]) -> &[u32] {
		self.reset(policy, parameter_count);

		self.assignments.clear();
		self.assignments.resize(values.len(), 0);

		self.assign_parameters(policy, parameter_count, values);
		self.collect_order(parameter_count, values);
		self.scan(values);

		&self.assignments
	}
}
