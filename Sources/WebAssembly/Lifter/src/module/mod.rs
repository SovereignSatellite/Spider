use alloc::sync::Arc;

use wasmparser::{
	ElementItems, ExternalKind, FunctionBody, Imports, Parser, Payload, SectionLimited, TableInit,
	TypeRef,
};

use web_assembly_control_flow::instruction::MemorySize;

pub use self::{
	bindings::ModuleBindings,
	constant_expression::ConstantExpression,
	initialization::build_initialization,
	plans::{
		DataModePlan, DataPlan, ElementItemPlan, ElementKindPlan, ElementPlan, ExportKind,
		ExportPlan, ImportKind, ImportPlan, MemoryPlan, ModulePlan, TablePlan,
	},
	types::TypeRegistry,
};

mod bindings;
mod constant_expression;
mod initialization;
mod plans;
mod types;

impl ImportKind {
	fn build(kind: TypeRef, types: &mut TypeRegistry) -> Self {
		match kind {
			TypeRef::Func(type_index) => {
				types.add_function(type_index);

				Self::Function
			}
			TypeRef::Table(_) => Self::Table,
			TypeRef::Memory(_) => Self::Memory,
			TypeRef::Global(_) => Self::Global,
			TypeRef::Tag(_) => unimplemented!("`Tag` imports"),
			TypeRef::FuncExact(_) => unimplemented!("`FuncExact` imports"),
		}
	}
}

impl ImportPlan {
	fn build(import: wasmparser::Import<'_>, types: &mut TypeRegistry) -> Self {
		Self {
			namespace: Arc::<str>::from(import.module),
			identifier: Arc::<str>::from(import.name),
			kind: ImportKind::build(import.ty, types),
		}
	}

	fn build_all(section: SectionLimited<'_, Imports<'_>>, types: &mut TypeRegistry) -> Vec<Self> {
		section
			.into_iter()
			.map(Result::unwrap)
			.map(|group| match group {
				Imports::Single(_, import) => Self::build(import, types),
				Imports::Compact1 { .. } => unimplemented!("`Compact1` imports"),
				Imports::Compact2 { .. } => unimplemented!("`Compact2` imports"),
			})
			.collect()
	}
}

impl TablePlan {
	fn build(table: &wasmparser::Table<'_>) -> Self {
		let initializer = match &table.init {
			TableInit::RefNull => None,
			TableInit::Expr(expression) => Some(ConstantExpression::parse(expression)),
		};

		Self {
			minimum: table.ty.initial.try_into().unwrap(),
			maximum: table
				.ty
				.maximum
				.map_or(u32::MAX, |maximum| maximum.try_into().unwrap()),
			initializer,
		}
	}

	fn build_all(section: SectionLimited<'_, wasmparser::Table<'_>>) -> Vec<Self> {
		section
			.into_iter()
			.map(Result::unwrap)
			.map(|table| Self::build(&table))
			.collect()
	}
}

impl MemoryPlan {
	fn build(memory: wasmparser::MemoryType) -> Self {
		let page = u32::try_from(MemorySize::PAGE_SIZE).unwrap();

		Self {
			minimum: u32::try_from(memory.initial).unwrap().saturating_mul(page),
			maximum: memory.maximum.map_or(u32::MAX, |maximum| {
				u32::try_from(maximum).unwrap().saturating_mul(page)
			}),
		}
	}

	fn build_all(section: SectionLimited<'_, wasmparser::MemoryType>) -> Vec<Self> {
		section
			.into_iter()
			.map(Result::unwrap)
			.map(Self::build)
			.collect()
	}
}

fn build_globals(section: SectionLimited<'_, wasmparser::Global<'_>>) -> Vec<ConstantExpression> {
	section
		.into_iter()
		.map(Result::unwrap)
		.map(|global| ConstantExpression::parse(&global.init_expr))
		.collect()
}

impl ExportKind {
	fn build(kind: ExternalKind) -> Self {
		match kind {
			ExternalKind::Func => Self::Function,
			ExternalKind::Table => Self::Table,
			ExternalKind::Memory => Self::Memory,
			ExternalKind::Global => Self::Global,
			ExternalKind::Tag => unimplemented!("`Tag`"),
			ExternalKind::FuncExact => unimplemented!("`FuncExact`"),
		}
	}
}

impl ExportPlan {
	fn build_all(section: SectionLimited<'_, wasmparser::Export<'_>>) -> Vec<Self> {
		section
			.into_iter()
			.map(Result::unwrap)
			.map(|export| Self {
				identifier: Arc::<str>::from(export.name),
				kind: ExportKind::build(export.kind),
				index: export.index,
			})
			.collect()
	}
}

fn build_element_items(items: ElementItems<'_>) -> Vec<ElementItemPlan> {
	match items {
		ElementItems::Functions(section) => section
			.into_iter()
			.map(Result::unwrap)
			.map(ElementItemPlan::Function)
			.collect(),
		ElementItems::Expressions(_, section) => section
			.into_iter()
			.map(Result::unwrap)
			.map(|expression| ElementItemPlan::Expression(ConstantExpression::parse(&expression)))
			.collect(),
	}
}

impl ElementKindPlan {
	fn build(kind: &wasmparser::ElementKind<'_>) -> Self {
		match kind {
			wasmparser::ElementKind::Passive => Self::Passive,
			wasmparser::ElementKind::Active {
				table_index,
				offset_expr: offset_expression,
			} => Self::Active {
				table: table_index.unwrap_or(0),
				offset: ConstantExpression::parse(offset_expression),
			},
			wasmparser::ElementKind::Declared => Self::Declared,
		}
	}
}

impl ElementPlan {
	fn build_all(section: SectionLimited<'_, wasmparser::Element<'_>>) -> Vec<Self> {
		section
			.into_iter()
			.map(Result::unwrap)
			.map(|element| Self {
				items: build_element_items(element.items),
				kind: ElementKindPlan::build(&element.kind),
			})
			.collect()
	}
}

impl DataModePlan {
	fn build(kind: &wasmparser::DataKind<'_>) -> Self {
		match kind {
			wasmparser::DataKind::Passive => Self::Passive,
			wasmparser::DataKind::Active {
				memory_index,
				offset_expr: offset_expression,
			} => Self::Active {
				memory: *memory_index,
				offset: ConstantExpression::parse(offset_expression),
			},
		}
	}
}

impl DataPlan {
	fn build_all(section: SectionLimited<'_, wasmparser::Data<'_>>) -> Vec<Self> {
		section
			.into_iter()
			.map(Result::unwrap)
			.map(|data| Self {
				bytes: Arc::<[u8]>::from(data.data),
				kind: DataModePlan::build(&data.kind),
			})
			.collect()
	}
}

pub struct ModuleDefinition<'data> {
	pub code: Vec<FunctionBody<'data>>,
	pub plan: ModulePlan,
}

impl<'data> ModuleDefinition<'data> {
	const fn new() -> Self {
		Self {
			code: Vec::new(),
			plan: ModulePlan::new(),
		}
	}

	fn handle_payload(&mut self, payload: Payload<'data>, types: &mut TypeRegistry) {
		match payload {
			Payload::Version { .. }
			| Payload::DataCountSection { .. }
			| Payload::CustomSection(_)
			| Payload::End(_) => {}

			Payload::TypeSection(section) => types.add_sub_types(section),
			Payload::ImportSection(section) => {
				self.plan.imports = ImportPlan::build_all(section, types);
			}
			Payload::FunctionSection(section) => types.add_functions(section),
			Payload::TableSection(section) => {
				self.plan.tables = TablePlan::build_all(section);
			}
			Payload::MemorySection(section) => {
				self.plan.memories = MemoryPlan::build_all(section);
			}
			Payload::TagSection(section) => {
				if section.count() != 0 {
					unimplemented!("`Tag`s are not supported yet");
				}
			}
			Payload::GlobalSection(section) => self.plan.globals = build_globals(section),
			Payload::ExportSection(section) => {
				self.plan.exports = ExportPlan::build_all(section);
			}
			Payload::StartSection { func: function, .. } => {
				self.plan.start = Some(function);
			}
			Payload::ElementSection(section) => {
				self.plan.elements = ElementPlan::build_all(section);
			}
			Payload::DataSection(section) => self.plan.datas = DataPlan::build_all(section),

			Payload::CodeSectionStart { count, .. } => {
				self.code.reserve_exact(count.try_into().unwrap());
			}
			Payload::CodeSectionEntry(body) => self.code.push(body),

			payload @ (Payload::UnknownSection { .. } | _) => unimplemented!("{payload:?}"),
		}
	}

	pub fn parse(binary: &'data [u8], types: &mut TypeRegistry) -> Self {
		let mut module = Self::new();
		types.clear();

		for payload in Parser::new(0).parse_all(binary).map(Result::unwrap) {
			module.handle_payload(payload, types);
		}

		module
	}
}
