use alloc::vec::Vec;
use control_flow_liveness::references::{Reference, ReferenceType};
use data_flow_graph::Link;
use wasmparser::{ExternalKind, TypeRef};

pub struct GlobalState {
	pub functions: Vec<Link>,
	pub tables: Vec<Link>,
	pub memories: Vec<Link>,
	pub globals: Vec<Link>,

	pub elements: Vec<Link>,
	pub datas: Vec<Link>,
}

impl GlobalState {
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

	pub fn retrieve_all_mutable(&self, results: &mut Vec<Link>) {
		results.extend_from_slice(&self.functions);
		results.extend_from_slice(&self.tables);
		results.extend_from_slice(&self.memories);
		results.extend_from_slice(&self.globals);
		results.extend_from_slice(&self.elements);
		results.extend_from_slice(&self.datas);
	}

	fn get_dependency(&self, reference: Reference) -> Link {
		let list = match reference.r#type {
			ReferenceType::Function => &self.functions,
			ReferenceType::Global => &self.globals,
			ReferenceType::Table => &self.tables,
			ReferenceType::Elements => &self.elements,
			ReferenceType::Memory => &self.memories,
			ReferenceType::Data => &self.datas,
		};

		list[usize::from(reference.id)]
	}

	pub fn get_dependencies(&self, references: &[Reference]) -> Vec<Link> {
		references
			.iter()
			.map(|&dependency| self.get_dependency(dependency))
			.collect()
	}

	pub fn get_external_kind(&self, external_kind: ExternalKind) -> &[Link] {
		match external_kind {
			ExternalKind::Func => &self.functions,
			ExternalKind::Table => &self.tables,
			ExternalKind::Memory => &self.memories,
			ExternalKind::Global => &self.globals,
			ExternalKind::Tag => unimplemented!("`Tag`"),
		}
	}

	pub fn get_mut_type_ref(&mut self, type_ref: TypeRef) -> &mut Vec<Link> {
		match type_ref {
			TypeRef::Func(_) => &mut self.functions,
			TypeRef::Table(_) => &mut self.tables,
			TypeRef::Memory(_) => &mut self.memories,
			TypeRef::Global(_) => &mut self.globals,
			TypeRef::Tag(_) => unimplemented!("`Tag`"),
		}
	}
}
