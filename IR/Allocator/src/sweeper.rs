//! The class sweeper: a single forward scan assigning registers to coalesced
//! classes in footprint start order, first fit by lowest id, with overflow
//! past the physical file onto spills.

use alloc::vec::Vec;

use crate::{classes::Classes, policy::Policy, value::Value};

#[derive(Clone, Copy)]
struct Slot {
	id: u32,
	mask: u8,
}

impl Slot {
	const fn accepts(self, kind: u8) -> bool {
		self.mask & kind == kind
	}
}

#[derive(Clone, Copy)]
struct Active {
	end: u32,
	register: u32,
	mask: u8,
}

/// Assigns registers to coalesced classes by a single forward scan.
pub struct Sweeper {
	order: Vec<u32>,
	free: Vec<Slot>,
	active: Vec<Active>,
	assignments: Vec<u32>,
	spill_cursor: u32,
}

impl Sweeper {
	/// Creates a new sweeper.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			order: Vec::new(),
			free: Vec::new(),
			active: Vec::new(),
			assignments: Vec::new(),
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
				.map(|(&mask, id)| Slot { id, mask }),
		);

		let physical_count = u32::try_from(policy.registers().len()).unwrap();

		self.spill_cursor = physical_count.max(u32::from(parameter_count));
	}

	fn insert_free(&mut self, slot: Slot) {
		let position = self.free.partition_point(|existing| existing.id < slot.id);

		self.free.insert(position, slot);
	}

	fn expire(&mut self, time: u32) {
		let mut index = 0;

		while index < self.active.len() {
			if self.active[index].end < time {
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

	fn take(&mut self, kind: u8) -> (u32, u8) {
		if let Some(position) = self.free.iter().position(|slot| slot.accepts(kind)) {
			let slot = self.free.remove(position);

			return (slot.id, slot.mask);
		}

		let id = self.spill_cursor;

		self.spill_cursor += 1;

		(id, u8::MAX)
	}

	// Register `index` is where the calling convention already placed parameter
	// `index`; a parameter beyond the physical file lands on a spill slot.
	fn activate_parameters(
		&mut self,
		policy: &dyn Policy,
		parameter_count: u16,
		classes: &Classes,
	) {
		for index in 0..u32::from(parameter_count) {
			let mask = policy
				.registers()
				.get(usize::try_from(index).unwrap())
				.copied()
				.unwrap_or(u8::MAX);

			debug_assert_eq!(
				classes.root_of(index),
				index,
				"parameter classes must stay rooted at their own value ids"
			);

			self.assignments[usize::try_from(index).unwrap()] = index;
			self.active.push(Active {
				end: classes.reach(index),
				register: index,
				mask,
			});
		}
	}

	fn collect_order(&mut self, parameter_count: u16, classes: &Classes) {
		self.order.clear();
		self.order.extend(
			(u32::from(parameter_count)..u32::try_from(classes.len()).unwrap())
				.filter(|&value| classes.root_of(value) == value),
		);
		self.order
			.sort_unstable_by_key(|&root| (classes.start(root), root));
	}

	fn scan(&mut self, classes: &Classes) {
		for index in 0..self.order.len() {
			let root = self.order[index];

			self.expire(classes.start(root));

			let (register, mask) = self.take(classes.kind(root));

			self.assignments[usize::try_from(root).unwrap()] = register;
			self.active.push(Active {
				end: classes.reach(root),
				register,
				mask,
			});
		}
	}

	fn spread(&mut self, classes: &Classes) {
		for value in 0..u32::try_from(classes.len()).unwrap() {
			let root = classes.root_of(value);

			self.assignments[usize::try_from(value).unwrap()] =
				self.assignments[usize::try_from(root).unwrap()];
		}
	}

	// Interfering values may share a register only within one class, where they
	// hold the same value; a coalescing bug that broke this would surface only
	// as a distant miscompile, so it is checked here.
	#[cfg(debug_assertions)]
	fn is_valid(&self, values: &[Value], classes: &Classes) -> bool {
		values.iter().enumerate().all(|(left, interval)| {
			values.iter().enumerate().take(left).all(|(right, other)| {
				let interferes = interval.start <= other.end && other.start <= interval.end;

				classes.root_of(u32::try_from(left).unwrap())
					== classes.root_of(u32::try_from(right).unwrap())
					|| self.assignments[left] != self.assignments[right]
					|| !interferes
			})
		})
	}

	#[cfg(not(debug_assertions))]
	#[expect(
		clippy::unused_self,
		reason = "the debug validator reads the assignments through self"
	)]
	const fn is_valid(&self, _values: &[Value], _classes: &Classes) -> bool {
		true
	}

	/// Assigns registers to every class by the forward scan and returns the
	/// per-value assignment table.
	pub fn run(
		&mut self,
		policy: &dyn Policy,
		parameter_count: u16,
		values: &[Value],
		classes: &mut Classes,
	) -> &[u32] {
		classes.flatten();

		self.reset(policy, parameter_count);
		self.assignments.clear();
		self.assignments.resize(classes.len(), 0);

		self.activate_parameters(policy, parameter_count, classes);
		self.collect_order(parameter_count, classes);
		self.scan(classes);
		self.spread(classes);

		debug_assert!(
			self.is_valid(values, classes),
			"coalescing left two interfering values sharing a register"
		);

		&self.assignments
	}
}
