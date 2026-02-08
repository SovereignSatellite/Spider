use alloc::vec::Vec;
use luajit_tree::expression::Local;

pub struct AssignmentSimplifier {
	assignments: Vec<(Local, Local)>,
}

impl AssignmentSimplifier {
	pub fn new(mut assignments: Vec<(Local, Local)>) -> Self {
		assignments.sort_unstable();

		Self { assignments }
	}

	fn find_first_assign(&self) -> Option<usize> {
		self.assignments.iter().position(|&(destination, _)| {
			self.assignments
				.iter()
				.all(|&(_, source)| destination != source)
		})
	}

	pub fn find_all_assigns<H>(&mut self, mut handler: H)
	where
		H: FnMut(Local, Local),
	{
		while let Some(index) = self.find_first_assign() {
			let (destination, source) = self.assignments.remove(index);

			handler(destination, source);
		}
	}

	fn find_first_swap(&self, next: (Local, Local), path: &mut Vec<Local>) {
		let (mut destination, mut source) = next;

		path.clear();

		loop {
			path.push(destination);

			(destination, source) = *self
				.assignments
				.iter()
				.find(|assignment| assignment.0 == source)
				.unwrap();

			if path.contains(&destination) {
				break;
			}
		}
	}

	pub fn find_all_swaps<H>(&mut self, mut handler: H)
	where
		H: FnMut(&[Local]),
	{
		let mut path = Vec::new();

		while let Some(&first) = self.assignments.first() {
			self.find_first_swap(first, &mut path);

			handler(&path);

			self.assignments
				.retain(|&assignment| !path.contains(&assignment.0));
		}
	}
}
