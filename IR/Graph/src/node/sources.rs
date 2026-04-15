#![expect(
	unused_variables,
	unused_mut,
	reason = "macro-generated visitors may not use all fields"
)]

use crate::Link;

use super::{
	Node,
	control::{
		Branch, BranchResults, Function, FunctionResults, Import, Match, Module, ModuleResults,
		Region, Repeat, RepeatResults,
	},
	simple::{
		Apply, Fence, GlobalGet, GlobalNew, GlobalSet, Identity, IntegerBinaryOperation,
		IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend, IntegerNarrow,
		IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden, Location, MemoryCopy,
		MemoryDrop, MemoryFill, MemoryGrow, MemoryLoad, MemoryNew, MemorySize, MemoryStore,
		NumberBinaryOperation, NumberCompareOperation, NumberNarrow, NumberTransmuteToInteger,
		NumberTruncateToInteger, NumberUnaryOperation, NumberWiden, RefIsNull, TableCopy,
		TableDrop, TableFill, TableGet, TableGrow, TableNew, TableSet, TableSize,
	},
};

macro_rules! for_each_visit {
	($self:ident, $visit:ident, $handler:ident) => {
		match $self {
			Self::Function(arc) => arc.lock().$visit($handler),
			Self::Match(arc) => arc.lock().$visit($handler),
			Self::Repeat(arc) => arc.lock().$visit($handler),

			Self::ModuleArguments(_)
			| Self::FunctionCaptures(_)
			| Self::FunctionArguments(_)
			| Self::BranchArguments(_)
			| Self::RepeatArguments(_)
			| Self::Trap
			| Self::Null
			| Self::I32(_)
			| Self::I64(_)
			| Self::F32(_)
			| Self::F64(_) => {}

			Self::ModuleResults(node) => node.$visit($handler),
			Self::FunctionResults(node) => node.$visit($handler),
			Self::BranchResults(node) => node.$visit($handler),
			Self::RepeatResults(node) => node.$visit($handler),

			Self::Import(node) => node.$visit($handler),
			Self::Host(host) => host.$visit(&mut $handler),

			Self::Identity(node) => node.$visit($handler),
			Self::Fence(node) => node.$visit($handler),
			Self::Apply(node) => node.$visit($handler),
			Self::RefIsNull(node) => node.$visit($handler),
			Self::IntegerUnaryOperation(node) => node.$visit($handler),
			Self::IntegerBinaryOperation(node) => node.$visit($handler),
			Self::IntegerCompareOperation(node) => node.$visit($handler),
			Self::IntegerNarrow(node) => node.$visit($handler),
			Self::IntegerWiden(node) => node.$visit($handler),
			Self::IntegerExtend(node) => node.$visit($handler),
			Self::IntegerConvertToNumber(node) => node.$visit($handler),
			Self::IntegerTransmuteToNumber(node) => node.$visit($handler),
			Self::NumberUnaryOperation(node) => node.$visit($handler),
			Self::NumberBinaryOperation(node) => node.$visit($handler),
			Self::NumberCompareOperation(node) => node.$visit($handler),
			Self::NumberNarrow(node) => node.$visit($handler),
			Self::NumberWiden(node) => node.$visit($handler),
			Self::NumberTruncateToInteger(node) => node.$visit($handler),
			Self::NumberTransmuteToInteger(node) => node.$visit($handler),
			Self::GlobalNew(node) => node.$visit($handler),
			Self::GlobalGet(node) => node.$visit($handler),
			Self::GlobalSet(node) => node.$visit($handler),
			Self::TableNew(node) => node.$visit($handler),
			Self::TableGet(node) => node.$visit($handler),
			Self::TableSet(node) => node.$visit($handler),
			Self::TableSize(node) => node.$visit($handler),
			Self::TableGrow(node) => node.$visit($handler),
			Self::TableFill(node) => node.$visit($handler),
			Self::TableCopy(node) => node.$visit($handler),
			Self::TableDrop(node) => node.$visit($handler),
			Self::MemoryNew(node) => node.$visit($handler),
			Self::MemoryLoad(node) => node.$visit($handler),
			Self::MemoryStore(node) => node.$visit($handler),
			Self::MemorySize(node) => node.$visit($handler),
			Self::MemoryGrow(node) => node.$visit($handler),
			Self::MemoryFill(node) => node.$visit($handler),
			Self::MemoryCopy(node) => node.$visit($handler),
			Self::MemoryDrop(node) => node.$visit($handler),
		}
	};
}

macro_rules! handle_field {
	($handler:ident, $name:ident, call) => {
		$handler($name);
	};
	($handler:ident, $name:ident, dereference_call) => {
		$handler(*$name);
	};
	($handler:ident, $name:ident, for_each, $($rest:tt)*) => {
		for item in $name {
			handle_field!($handler, item, $($rest)*);
		}
	};
	($handler:ident, $name:ident, method, $method:ident) => {
		$name.$method(&mut $handler);
	};
}

macro_rules! handle_argument_source {
	($handler:ident, $name:ident, ignore) => {};
	($handler:ident, $name:ident, link) => {
		handle_field!($handler, $name, dereference_call)
	};
	($handler:ident, $name:ident, link_list) => {
		handle_field!($handler, $name, for_each, dereference_call)
	};
	($handler:ident, $name:ident, method) => {
		handle_field!($handler, $name, method, for_each_outer)
	};
}

macro_rules! handle_mut_argument_source {
	($handler:ident, $name:ident, ignore) => {};
	($handler:ident, $name:ident, link) => {
		handle_field!($handler, $name, call)
	};
	($handler:ident, $name:ident, link_list) => {
		handle_field!($handler, $name, for_each, call)
	};
	($handler:ident, $name:ident, method) => {
		handle_field!($handler, $name, method, for_each_mut_outer)
	};
}

macro_rules! handle_visitor {
	($name:ident, $type:ty, $visitor:ident, ( $( ($field:ident, $variant:ident) ),* )) => {
        fn $name<H: FnMut($type)>(&self, mut handler: H) {
        	let Self { $($field),* } = self;

			$(
				$visitor!(handler, $field, $variant)
			);*;
        }
    };
}

macro_rules! handle_mut_visitor {
	($name:ident, $type:ty, $visitor:ident, ( $( ($field:ident, $variant:ident) ),* )) => {
		fn $name<H: FnMut(&mut $type)>(&mut self, mut handler: H) {
        	let Self { $($field),* } = self;

			$(
				$visitor!(handler, $field, $variant)
			);*;
        }
    };
}

macro_rules! handle_sources {
	($( ($field:ident, $action:ident) ),*) => {
		handle_visitor!(for_each_outer, Link, handle_argument_source, ( $( ($field, $action) ),* ));
		handle_mut_visitor!(for_each_mut_outer, Link, handle_mut_argument_source, ( $( ($field, $action) ),* ));
	};
}

impl ModuleResults {
	fn for_each_outer<H: FnMut(Link)>(&self, mut handler: H) {
		handler(self.state);

		for export in &self.exports {
			handler(export.reference);
		}
	}

	fn for_each_mut_outer<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		handler(&mut self.state);

		for export in &mut self.exports {
			handler(&mut export.reference);
		}
	}
}

impl FunctionResults {
	handle_sources!((parent, ignore), (sources, link_list));
}

impl BranchResults {
	handle_sources!((parent, ignore), (sources, link_list));
}

impl RepeatResults {
	handle_sources!((parent, ignore), (sources, link_list), (condition, link));
}

impl Import {
	handle_sources!(
		(environment, link),
		(namespace, ignore),
		(identifier, ignore)
	);
}

impl Identity {
	handle_sources!((sources, link_list));
}

impl Fence {
	handle_sources!((sources, link_list));
}

impl Apply {
	handle_sources!((function, link), (arguments, link_list), (results, ignore));
}

impl RefIsNull {
	handle_sources!((source, link));
}

impl IntegerUnaryOperation {
	handle_sources!((source, link), (kind, ignore), (operator, ignore));
}

impl IntegerBinaryOperation {
	handle_sources!((lhs, link), (rhs, link), (kind, ignore), (operator, ignore));
}

impl IntegerCompareOperation {
	handle_sources!((lhs, link), (rhs, link), (kind, ignore), (operator, ignore));
}

impl IntegerNarrow {
	handle_sources!((source, link));
}

impl IntegerWiden {
	handle_sources!((source, link));
}

impl IntegerExtend {
	handle_sources!((source, link), (kind, ignore));
}

impl IntegerConvertToNumber {
	handle_sources!(
		(source, link),
		(signed, ignore),
		(to, ignore),
		(from, ignore)
	);
}

impl IntegerTransmuteToNumber {
	handle_sources!((source, link), (from, ignore));
}

impl NumberUnaryOperation {
	handle_sources!((source, link), (kind, ignore), (operator, ignore));
}

impl NumberBinaryOperation {
	handle_sources!((lhs, link), (rhs, link), (kind, ignore), (operator, ignore));
}

impl NumberCompareOperation {
	handle_sources!((lhs, link), (rhs, link), (kind, ignore), (operator, ignore));
}

impl NumberTruncateToInteger {
	handle_sources!(
		(source, link),
		(signed, ignore),
		(saturate, ignore),
		(to, ignore),
		(from, ignore)
	);
}

impl NumberTransmuteToInteger {
	handle_sources!((source, link), (from, ignore));
}

impl NumberNarrow {
	handle_sources!((source, link));
}

impl NumberWiden {
	handle_sources!((source, link));
}

impl Location {
	handle_sources!((reference, link), (offset, link));
}

impl GlobalNew {
	handle_sources!((initializer, link));
}

impl GlobalGet {
	handle_sources!((source, link));
}

impl GlobalSet {
	handle_sources!((destination, link), (source, link));
}

impl TableNew {
	fn for_each_outer<H: FnMut(Link)>(&self, mut handler: H) {
		let Self { initializer, .. } = self;

		for item in initializer {
			handler(item.0);
		}
	}

	fn for_each_mut_outer<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { initializer, .. } = self;

		for item in initializer {
			handler(&mut item.0);
		}
	}
}

impl TableGet {
	handle_sources!((source, method));
}

impl TableSet {
	handle_sources!((destination, method), (source, link));
}

impl TableSize {
	handle_sources!((source, link));
}

impl TableGrow {
	handle_sources!((destination, link), (initializer, link), (size, link));
}

impl TableFill {
	handle_sources!((destination, method), (source, link), (size, link));
}

impl TableCopy {
	handle_sources!((destination, method), (source, method), (size, link));
}

impl TableDrop {
	handle_sources!((source, link));
}

impl MemoryNew {
	handle_sources!((initializer, ignore), (minimum, ignore), (maximum, ignore));
}

impl MemoryLoad {
	handle_sources!((source, method), (kind, ignore));
}

impl MemoryStore {
	handle_sources!((destination, method), (source, link), (kind, ignore));
}

impl MemorySize {
	handle_sources!((source, link));
}

impl MemoryGrow {
	handle_sources!((destination, link), (size, link));
}

impl MemoryFill {
	handle_sources!((destination, method), (byte, link), (size, link));
}

impl MemoryCopy {
	handle_sources!((destination, method), (source, method), (size, link));
}

impl MemoryDrop {
	handle_sources!((source, link));
}

impl Module {
	fn nodes(&self) -> &[Node] {
		&self.nodes
	}

	const fn nodes_mut(&mut self) -> &mut Vec<Node> {
		&mut self.nodes
	}

	/// Returns the index of the results boundary node.
	#[must_use]
	pub fn results_index(&self) -> usize {
		self.nodes
			.iter()
			.rposition(|node| matches!(node, Node::ModuleResults(_)))
			.unwrap_or_else(|| unreachable!())
	}

	/// Returns a reference to the results boundary node.
	#[must_use]
	pub fn results(&self) -> &ModuleResults {
		if let Node::ModuleResults(results) = &self.nodes[self.results_index()] {
			results
		} else {
			unreachable!()
		}
	}

	/// Returns a mutable reference to the results boundary node.
	pub fn results_mut(&mut self) -> &mut ModuleResults {
		let index = self.results_index();

		if let Node::ModuleResults(results) = &mut self.nodes[index] {
			results
		} else {
			unreachable!()
		}
	}

	fn for_each_root<H: FnMut(u32)>(&self, mut handler: H) {
		let result = u32::try_from(self.results_index()).unwrap_or_else(|_| unreachable!());

		handler(Self::ARGUMENTS_ID);
		handler(result);
	}
}

impl Function {
	/// Visits each outer link (captures).
	pub fn for_each_outer<H: FnMut(Link)>(&self, mut handler: H) {
		for &link in &self.captures {
			handler(link);
		}
	}

	/// Mutably visits each outer link (captures).
	pub fn for_each_mut_outer<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		for link in &mut self.captures {
			handler(link);
		}
	}

	fn nodes(&self) -> &[Node] {
		&self.nodes
	}

	const fn nodes_mut(&mut self) -> &mut Vec<Node> {
		&mut self.nodes
	}

	/// Returns the index of the results boundary node.
	#[must_use]
	pub fn results_index(&self) -> usize {
		self.nodes
			.iter()
			.rposition(|node| matches!(node, Node::FunctionResults(_)))
			.unwrap_or_else(|| unreachable!())
	}

	/// Returns a reference to the results boundary node.
	#[must_use]
	pub fn results(&self) -> &FunctionResults {
		if let Node::FunctionResults(results) = &self.nodes[self.results_index()] {
			results
		} else {
			unreachable!()
		}
	}

	/// Returns a mutable reference to the results boundary node.
	pub fn results_mut(&mut self) -> &mut FunctionResults {
		let index = self.results_index();

		if let Node::FunctionResults(results) = &mut self.nodes[index] {
			results
		} else {
			unreachable!()
		}
	}

	fn for_each_root<H: FnMut(u32)>(&self, mut handler: H) {
		let result = u32::try_from(self.results_index()).unwrap_or_else(|_| unreachable!());

		handler(Self::CAPTURES_ID);
		handler(Self::ARGUMENTS_ID);
		handler(result);
	}
}

impl Match {
	/// Visits each outer link (arguments, condition).
	pub fn for_each_outer<H: FnMut(Link)>(&self, mut handler: H) {
		for &link in &self.arguments {
			handler(link);
		}

		handler(self.condition);
	}

	/// Mutably visits each outer link (arguments, condition).
	pub fn for_each_mut_outer<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		for link in &mut self.arguments {
			handler(link);
		}

		handler(&mut self.condition);
	}
}

impl Branch {
	fn nodes(&self) -> &[Node] {
		&self.nodes
	}

	const fn nodes_mut(&mut self) -> &mut Vec<Node> {
		&mut self.nodes
	}

	/// Returns the index of the results boundary node.
	#[must_use]
	pub fn results_index(&self) -> usize {
		self.nodes
			.iter()
			.rposition(|node| matches!(node, Node::BranchResults(_)))
			.unwrap_or_else(|| unreachable!())
	}

	/// Returns a reference to the results boundary node.
	#[must_use]
	pub fn results(&self) -> &BranchResults {
		if let Node::BranchResults(results) = &self.nodes[self.results_index()] {
			results
		} else {
			unreachable!()
		}
	}

	/// Returns a mutable reference to the results boundary node.
	pub fn results_mut(&mut self) -> &mut BranchResults {
		let index = self.results_index();

		if let Node::BranchResults(results) = &mut self.nodes[index] {
			results
		} else {
			unreachable!()
		}
	}

	fn for_each_root<H: FnMut(u32)>(&self, mut handler: H) {
		let result = u32::try_from(self.results_index()).unwrap_or_else(|_| unreachable!());

		handler(Self::ARGUMENTS_ID);
		handler(result);
	}
}

impl Repeat {
	/// Visits each outer link (arguments).
	pub fn for_each_outer<H: FnMut(Link)>(&self, mut handler: H) {
		for &link in &self.arguments {
			handler(link);
		}
	}

	/// Mutably visits each outer link (arguments).
	pub fn for_each_mut_outer<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		for link in &mut self.arguments {
			handler(link);
		}
	}

	fn nodes(&self) -> &[Node] {
		&self.nodes
	}

	const fn nodes_mut(&mut self) -> &mut Vec<Node> {
		&mut self.nodes
	}

	/// Returns the index of the results boundary node.
	#[must_use]
	pub fn results_index(&self) -> usize {
		self.nodes
			.iter()
			.rposition(|node| matches!(node, Node::RepeatResults(_)))
			.unwrap_or_else(|| unreachable!())
	}

	/// Returns a reference to the results boundary node.
	#[must_use]
	pub fn results(&self) -> &RepeatResults {
		if let Node::RepeatResults(results) = &self.nodes[self.results_index()] {
			results
		} else {
			unreachable!()
		}
	}

	/// Returns a mutable reference to the results boundary node.
	pub fn results_mut(&mut self) -> &mut RepeatResults {
		let index = self.results_index();

		if let Node::RepeatResults(results) = &mut self.nodes[index] {
			results
		} else {
			unreachable!()
		}
	}

	fn for_each_root<H: FnMut(u32)>(&self, mut handler: H) {
		let result = u32::try_from(self.results_index()).unwrap_or_else(|_| unreachable!());

		handler(Self::ARGUMENTS_ID);
		handler(result);
	}
}

impl Region {
	/// Returns a reference to the region's nodes.
	#[must_use]
	pub fn nodes(&self) -> &[Node] {
		match self {
			Self::Module(region) => region.nodes(),
			Self::Function(region) => region.nodes(),
			Self::Branch(region) => region.nodes(),
			Self::Repeat(region) => region.nodes(),
		}
	}

	/// Returns a mutable reference to the region's nodes.
	pub fn nodes_mut(&mut self) -> &mut Vec<Node> {
		match self {
			Self::Module(region) => region.nodes_mut(),
			Self::Function(region) => region.nodes_mut(),
			Self::Branch(region) => region.nodes_mut(),
			Self::Repeat(region) => region.nodes_mut(),
		}
	}

	/// Visits each root node index in the region.
	pub fn for_each_root<H: FnMut(u32)>(&self, handler: H) {
		match self {
			Self::Module(region) => region.for_each_root(handler),
			Self::Function(region) => region.for_each_root(handler),
			Self::Branch(region) => region.for_each_root(handler),
			Self::Repeat(region) => region.for_each_root(handler),
		}
	}
}

impl Node {
	/// Visits each outer link (arguments from the parent region's perspective).
	pub fn for_each_outer<H: FnMut(Link)>(&self, mut handler: H) {
		for_each_visit!(self, for_each_outer, handler);
	}

	/// Mutably visits each outer link (arguments from the parent region's perspective).
	pub fn for_each_mut_outer<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		for_each_visit!(self, for_each_mut_outer, handler);
	}
}
