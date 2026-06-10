use alloc::sync::Arc;

use super::constant_expression::ConstantExpression;

pub struct ImportPlan {
	pub namespace: Arc<str>,
	pub identifier: Arc<str>,
	pub kind: ImportKind,
}

#[derive(Clone, Copy)]
pub enum ImportKind {
	Function { type_index: u32 },
	Table,
	Memory,
	Global,
}

pub struct ExportPlan {
	pub identifier: Arc<str>,
	pub kind: ExportKind,
	pub index: u32,
}

#[derive(Clone, Copy)]
pub enum ExportKind {
	Function,
	Table,
	Memory,
	Global,
}

#[derive(Clone, Copy)]
pub struct TablePlan {
	pub minimum: u32,
	pub maximum: u32,
	pub initializer: Option<ConstantExpression>,
}

#[derive(Clone, Copy)]
pub struct MemoryPlan {
	pub minimum: u32,
	pub maximum: u32,
}

pub struct ElementPlan {
	pub items: Vec<ElementItemPlan>,
	pub kind: ElementKindPlan,
}

#[derive(Clone, Copy)]
pub enum ElementItemPlan {
	Function(u32),
	Expression(ConstantExpression),
}

#[derive(Clone, Copy)]
pub enum ElementKindPlan {
	Active {
		table: u32,
		offset: ConstantExpression,
	},
	Passive,
	Declared,
}

pub struct DataPlan {
	pub bytes: Arc<[u8]>,
	pub kind: DataModePlan,
}

#[derive(Clone, Copy)]
pub enum DataModePlan {
	Passive,
	Active {
		memory: u32,
		offset: ConstantExpression,
	},
}

pub struct EnvironmentPlan {
	pub imports: Vec<ImportPlan>,
	pub globals: Vec<ConstantExpression>,
	pub tables: Vec<TablePlan>,
	pub memories: Vec<MemoryPlan>,
	pub elements: Vec<ElementPlan>,
	pub datas: Vec<DataPlan>,
	pub exports: Vec<ExportPlan>,
	pub start: Option<u32>,
}

impl EnvironmentPlan {
	pub const fn new() -> Self {
		Self {
			imports: Vec::new(),
			globals: Vec::new(),
			tables: Vec::new(),
			memories: Vec::new(),
			elements: Vec::new(),
			datas: Vec::new(),
			exports: Vec::new(),
			start: None,
		}
	}

	fn count_imports(&self, predicate: impl Fn(&ImportKind) -> bool) -> usize {
		self.imports
			.iter()
			.filter(|import| predicate(&import.kind))
			.count()
	}

	#[must_use]
	pub fn imported_table_count(&self) -> usize {
		self.count_imports(|kind| matches!(kind, ImportKind::Table))
	}

	#[must_use]
	pub fn imported_global_count(&self) -> usize {
		self.count_imports(|kind| matches!(kind, ImportKind::Global))
	}
}
