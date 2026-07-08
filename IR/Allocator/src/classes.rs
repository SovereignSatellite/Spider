//! Value classes: a union-find whose classes carry disjoint interval footprints.

use alloc::vec::Vec;

use crate::value::{self, Value};

const NIL: u32 = u32::MAX;

#[derive(Clone, Copy)]
struct Run {
	start: u32,
	end: u32,
	next: u32,
}

#[derive(Clone, Copy)]
struct Payload {
	head: u32,
	reach: u32,
	pin: u32,
	kind: u8,
}

/// The coalescing classes over one function's values: a union-find whose every
/// class carries the disjoint, start-sorted intervals its members occupy, the
/// register a parameter pins it to, and the union of its members' kind masks.
pub struct Classes {
	parents: Vec<u32>,
	payloads: Vec<Payload>,
	runs: Vec<Run>,
}

impl Classes {
	/// Creates an empty class table.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			parents: Vec::new(),
			payloads: Vec::new(),
			runs: Vec::new(),
		}
	}

	/// Rebuilds the table with one singleton class per value, pinning the first
	/// `parameter_count` classes to their own indices.
	pub fn reset(&mut self, values: &[Value], parameter_count: u16) {
		self.parents.clear();
		self.parents.extend(0..u32::try_from(values.len()).unwrap());

		self.payloads.clear();
		self.runs.clear();

		for (id, value) in values.iter().enumerate() {
			let pin = if id < usize::from(parameter_count) {
				u32::try_from(id).unwrap()
			} else {
				value::NONE
			};

			self.payloads.push(Payload {
				head: u32::try_from(id).unwrap(),
				reach: value.end,
				pin,
				kind: value.kind,
			});
			self.runs.push(Run {
				start: value.start,
				end: value.end,
				next: NIL,
			});
		}
	}

	/// Returns the number of values in the table.
	#[must_use]
	pub const fn len(&self) -> usize {
		self.parents.len()
	}

	/// Returns the representative of the value's class, halving the path walked.
	pub fn find(&mut self, mut value: u32) -> u32 {
		loop {
			let parent = self.parents[usize::try_from(value).unwrap()];

			if parent == value {
				return value;
			}

			let grandparent = self.parents[usize::try_from(parent).unwrap()];

			self.parents[usize::try_from(value).unwrap()] = grandparent;
			value = grandparent;
		}
	}

	/// Returns the register the class is pinned to, or [`value::NONE`].
	#[must_use]
	pub fn pin(&self, root: u32) -> u32 {
		self.payloads[usize::try_from(root).unwrap()].pin
	}

	/// Returns the union of the class members' kind masks.
	#[must_use]
	pub fn kind(&self, root: u32) -> u8 {
		self.payloads[usize::try_from(root).unwrap()].kind
	}

	/// Returns the first time point the class occupies.
	#[must_use]
	pub fn start(&self, root: u32) -> u32 {
		let head = self.payloads[usize::try_from(root).unwrap()].head;

		self.runs[usize::try_from(head).unwrap()].start
	}

	/// Returns the last time point the class occupies.
	#[must_use]
	pub fn reach(&self, root: u32) -> u32 {
		self.payloads[usize::try_from(root).unwrap()].reach
	}

	/// Returns whether the two classes' footprints share no time point.
	#[must_use]
	pub fn is_disjoint(&self, left: u32, right: u32) -> bool {
		let mut first = self.payloads[usize::try_from(left).unwrap()].head;
		let mut second = self.payloads[usize::try_from(right).unwrap()].head;

		while first != NIL && second != NIL {
			let left_run = self.runs[usize::try_from(first).unwrap()];
			let right_run = self.runs[usize::try_from(second).unwrap()];

			if left_run.start <= right_run.end && right_run.start <= left_run.end {
				return false;
			}

			if left_run.end < right_run.end {
				first = left_run.next;
			} else {
				second = right_run.next;
			}
		}

		true
	}

	/// Absorbs the `from` class into the `into` class, splicing footprints and
	/// keeping whichever pin exists.
	pub fn union(&mut self, into: u32, from: u32) {
		debug_assert_ne!(into, from, "a class cannot absorb itself");

		let absorbed = self.payloads[usize::try_from(from).unwrap()];

		self.parents[usize::try_from(from).unwrap()] = into;

		let payload = &mut self.payloads[usize::try_from(into).unwrap()];

		payload.reach = payload.reach.max(absorbed.reach);
		payload.kind |= absorbed.kind;

		if payload.pin == value::NONE {
			payload.pin = absorbed.pin;
		}

		let head = payload.head;

		self.payloads[usize::try_from(into).unwrap()].head = self.splice(head, absorbed.head);
	}

	// Merge two start-sorted disjoint chains into one, folding runs that touch or
	// overlap; the footprint stays disjoint and start-sorted.
	fn splice(&mut self, left: u32, right: u32) -> u32 {
		let (head, mut first, mut second) = self.split_heads(left, right);
		let mut tail = head;

		while first != NIL || second != NIL {
			let take_first = second == NIL
				|| (first != NIL
					&& self.runs[usize::try_from(first).unwrap()].start
						<= self.runs[usize::try_from(second).unwrap()].start);
			let cursor = if take_first { &mut first } else { &mut second };
			let node = *cursor;

			*cursor = self.runs[usize::try_from(node).unwrap()].next;

			if self.append(tail, node) {
				tail = node;
			}
		}

		head
	}

	// Detach the earlier-starting head to seed the merged chain; its
	// continuation joins the other chain as a walk cursor.
	fn split_heads(&mut self, left: u32, right: u32) -> (u32, u32, u32) {
		let left_start = self.runs[usize::try_from(left).unwrap()].start;
		let right_start = self.runs[usize::try_from(right).unwrap()].start;
		let (head, other) = if left_start <= right_start {
			(left, right)
		} else {
			(right, left)
		};
		let continuation = self.runs[usize::try_from(head).unwrap()].next;

		self.runs[usize::try_from(head).unwrap()].next = NIL;

		(head, continuation, other)
	}

	// Fold the node into the tail when they touch or overlap, else chain it;
	// returns whether the node became the new tail.
	fn append(&mut self, tail: u32, node: u32) -> bool {
		let Run { start, end, .. } = self.runs[usize::try_from(node).unwrap()];
		let tail_run = &mut self.runs[usize::try_from(tail).unwrap()];

		if start <= tail_run.end {
			tail_run.end = tail_run.end.max(end);

			return false;
		}

		tail_run.next = node;
		self.runs[usize::try_from(node).unwrap()].next = NIL;

		true
	}

	/// Rewrites every parent to its class representative, so [`Self::root_of`]
	/// answers without mutation.
	pub fn flatten(&mut self) {
		for value in 0..u32::try_from(self.parents.len()).unwrap() {
			let root = self.find(value);

			self.parents[usize::try_from(value).unwrap()] = root;
		}
	}

	/// Returns the flattened representative of the value's class.
	///
	/// # Panics
	///
	/// Panics in debug builds if [`Self::flatten`] has not run since the last
	/// union.
	#[must_use]
	pub fn root_of(&self, value: u32) -> u32 {
		let root = self.parents[usize::try_from(value).unwrap()];

		debug_assert_eq!(
			self.parents[usize::try_from(root).unwrap()],
			root,
			"roots must be flattened before direct reads"
		);

		root
	}
}

impl Default for Classes {
	fn default() -> Self {
		Self::new()
	}
}
