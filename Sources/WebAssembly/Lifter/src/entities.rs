use ir_graph::{Link, Node, operation::MutableGet};
use web_assembly_graph::instruction::{Reference, ReferenceType};

pub struct Entities {
	pub functions: Vec<Link>,
	pub tables: Vec<Link>,
	pub memories: Vec<Link>,
	pub globals: Vec<Link>,

	pub elements: Vec<Link>,
	pub datas: Vec<Link>,
}

impl Entities {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			functions: Vec::new(),
			tables: Vec::new(),
			memories: Vec::new(),
			globals: Vec::new(),

			elements: Vec::new(),
			datas: Vec::new(),
		}
	}

	pub fn clear(&mut self) {
		self.functions.clear();
		self.tables.clear();
		self.memories.clear();
		self.globals.clear();

		self.elements.clear();
		self.datas.clear();
	}

	pub fn collect_states_into(&self, results: &mut Vec<Link>) {
		results.extend_from_slice(&self.functions);
		results.extend_from_slice(&self.tables);
		results.extend_from_slice(&self.memories);
		results.extend_from_slice(&self.globals);
		results.extend_from_slice(&self.elements);
		results.extend_from_slice(&self.datas);
	}

	pub fn emit_function_reference(&self, nodes: &mut Vec<Node>, index: u32) -> Link {
		let Ok(index) = usize::try_from(index) else {
			unreachable!()
		};

		MutableGet::add_into(nodes, self.functions[index]).0
	}

	fn get_dependency(&self, reference: Reference) -> Link {
		let id = usize::from(reference.id);

		match reference.kind {
			ReferenceType::Function => self.functions[id],
			ReferenceType::Global => self.globals[id],
			ReferenceType::Table => self.tables[id],
			ReferenceType::Elements => self.elements[id],
			ReferenceType::Memory => self.memories[id],
			ReferenceType::Data => self.datas[id],
		}
	}

	pub fn get_dependencies(&self, references: &[Reference]) -> Vec<Link> {
		references
			.iter()
			.map(|&dependency| self.get_dependency(dependency))
			.collect()
	}
}
