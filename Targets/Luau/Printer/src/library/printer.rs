use std::io::{Result, Write};

use hashbrown::HashSet;

use super::sections::{Section, Sections};

pub struct Printer {
	references: Vec<&'static str>,
	expanded: HashSet<&'static str>,
}

impl Printer {
	#[must_use]
	pub fn new() -> Self {
		Self {
			references: Vec::new(),
			expanded: HashSet::new(),
		}
	}

	fn recursively_expand(&mut self, name: &'static str, sections: &Sections) {
		if !self.expanded.insert(name) {
			return;
		}

		for name in &sections.find(name).references {
			self.recursively_expand(name, sections);
		}

		self.references.push(name);
	}

	pub fn resolve(&mut self, names: &[&'static str], sections: &Sections) {
		self.references.clear();
		self.expanded.clear();

		for &name in names {
			self.recursively_expand(name, sections);
		}
	}

	pub fn print(&self, sections: &Sections, out: &mut dyn Write) -> Result<()> {
		self.references.iter().try_for_each(|&name| {
			let Section { contents, .. } = sections.find(name);

			writeln!(out, "{contents}\n")
		})
	}
}

impl Default for Printer {
	fn default() -> Self {
		Self::new()
	}
}
