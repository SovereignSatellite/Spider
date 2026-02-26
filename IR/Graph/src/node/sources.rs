use crate::{
	Link, Node,
	node::{
		control::{
			Export, GammaIn, GammaOut, Import, LambdaIn, LambdaOut, OmegaIn, OmegaOut, RegionIn,
			RegionOut, ThetaIn, ThetaOut,
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
	},
};

macro_rules! for_each_visit {
	($self:ident, $visit:ident, $handler:ident) => {
		match $self {
			Self::LambdaIn(node) => node.$visit($handler),
			Self::LambdaOut(node) => node.$visit($handler),
			Self::RegionIn(node) => node.$visit($handler),
			Self::RegionOut(node) => node.$visit($handler),
			Self::GammaIn(node) => node.$visit($handler),
			Self::GammaOut(node) => node.$visit($handler),
			Self::ThetaIn(node) => node.$visit($handler),
			Self::ThetaOut(node) => node.$visit($handler),
			Self::OmegaIn(node) => node.$visit($handler),
			Self::OmegaOut(node) => node.$visit($handler),
			Self::Import(node) => node.$visit($handler),
			Self::Host(host) => host.$visit(&mut $handler),
			Self::Trap | Self::Null | Self::I32(_) | Self::I64(_) | Self::F32(_) | Self::F64(_) => {
			}

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
	($handler:ident, $name:ident, first_call) => {
		$handler($name.0);
	};
	($handler:ident, $name:ident, first_mut_call) => {
		$handler(&mut $name.0);
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

macro_rules! handle_id_source {
	($handler:ident, $name:ident, ignore) => {};
	($handler:ident, $name:ident, id) => {
		handle_field!($handler, $name, dereference_call)
	};
	($handler:ident, $name:ident, id_list) => {
		handle_field!($handler, $name, for_each, dereference_call)
	};
	($handler:ident, $name:ident, link) => {
		handle_field!($handler, $name, first_call)
	};
	($handler:ident, $name:ident, link_list) => {
		handle_field!($handler, $name, for_each, first_call)
	};
	($handler:ident, $name:ident, method) => {
		handle_field!($handler, $name, method, for_each_id)
	};
	($handler:ident, $name:ident, method_list) => {
		handle_field!($handler, $name, for_each, method, for_each_id)
	};
}

macro_rules! handle_mut_id_source {
	($handler:ident, $name:ident, ignore) => {};
	($handler:ident, $name:ident, id) => {
		handle_field!($handler, $name, call)
	};
	($handler:ident, $name:ident, id_list) => {
		handle_field!($handler, $name, for_each, call)
	};
	($handler:ident, $name:ident, link) => {
		handle_field!($handler, $name, first_mut_call)
	};
	($handler:ident, $name:ident, link_list) => {
		handle_field!($handler, $name, for_each, first_mut_call)
	};
	($handler:ident, $name:ident, method) => {
		handle_field!($handler, $name, method, for_each_mut_id)
	};
	($handler:ident, $name:ident, method_list) => {
		handle_field!($handler, $name, for_each, method, for_each_mut_id)
	};
}

macro_rules! handle_argument_source {
	($handler:ident, $name:ident, ignore) => {};
	($handler:ident, $name:ident, id) => {};
	($handler:ident, $name:ident, id_list) => {};
	($handler:ident, $name:ident, link) => {
		handle_field!($handler, $name, dereference_call)
	};
	($handler:ident, $name:ident, link_list) => {
		handle_field!($handler, $name, for_each, dereference_call)
	};
	($handler:ident, $name:ident, method) => {
		handle_field!($handler, $name, method, for_each_argument)
	};
	($handler:ident, $name:ident, method_list) => {
		handle_field!($handler, $name, for_each, method, for_each_argument)
	};
}

macro_rules! handle_mut_argument_source {
	($handler:ident, $name:ident, ignore) => {};
	($handler:ident, $name:ident, id) => {};
	($handler:ident, $name:ident, id_list) => {};
	($handler:ident, $name:ident, link) => {
		handle_field!($handler, $name, call)
	};
	($handler:ident, $name:ident, link_list) => {
		handle_field!($handler, $name, for_each, call)
	};
	($handler:ident, $name:ident, method) => {
		handle_field!($handler, $name, method, for_each_mut_argument)
	};
	($handler:ident, $name:ident, method_list) => {
		handle_field!($handler, $name, for_each, method, for_each_mut_argument)
	};
}

macro_rules! handle_visitor {
	($name:ident, $type:ty, $visitor:ident, ( $( ($field:ident, $variant:ident) ),* )) => {
		#[allow(unused_variables, unused_mut)]
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
		#[allow(unused_variables, unused_mut)]
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
		handle_visitor!(for_each_id, u32, handle_id_source, ( $( ($field, $action) ),* ));
		handle_visitor!(for_each_argument, Link, handle_argument_source, ( $( ($field, $action) ),* ));

		handle_mut_visitor!(for_each_mut_id, u32, handle_mut_id_source, ( $( ($field, $action) ),* ));
		handle_mut_visitor!(for_each_mut_argument, Link, handle_mut_argument_source, ( $( ($field, $action) ),* ));
	};
}

macro_rules! handle_requirements {
	($( ($field:ident, $action:ident) ),*) => {
		handle_visitor!(for_each_requirement, u32, handle_id_source, ( $( ($field, $action) ),* ));
	}
}

impl LambdaIn {
	handle_sources!((output, id), (r#type, ignore), (dependencies, link_list));
}

impl LambdaOut {
	handle_requirements!((input, id), (results, ignore));
	handle_sources!((input, id), (results, link_list));
}

impl RegionIn {
	handle_requirements!((input, id), (output, ignore));
	handle_sources!((input, id), (output, id));
}

impl RegionOut {
	handle_requirements!((input, id), (output, ignore), (results, ignore));
	handle_sources!((input, id), (output, id), (results, link_list));
}

impl GammaIn {
	handle_sources!((output, id), (arguments, link_list), (condition, link));
}

impl GammaOut {
	handle_requirements!((input, id), (regions, id_list));
	handle_sources!((input, id), (regions, id_list));
}

impl ThetaIn {
	handle_sources!((output, id), (arguments, link_list));
}

impl ThetaOut {
	handle_requirements!((input, id), (results, ignore), (condition, ignore));
	handle_sources!((input, id), (results, link_list), (condition, link));
}

impl OmegaIn {
	handle_sources!((output, id));
}

impl Export {
	handle_sources!((identifier, ignore), (reference, link));
}

impl OmegaOut {
	handle_requirements!((input, id), (state, ignore), (exports, ignore));
	handle_sources!((input, id), (state, link), (exports, method_list));
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
	handle_sources!(
		(function, link),
		(arguments, link_list),
		(results, ignore),
		(states, ignore)
	);
}

impl RefIsNull {
	handle_sources!((source, link));
}

impl IntegerUnaryOperation {
	handle_sources!((source, link), (r#type, ignore), (operator, ignore));
}

impl IntegerBinaryOperation {
	handle_sources!(
		(lhs, link),
		(rhs, link),
		(r#type, ignore),
		(operator, ignore)
	);
}

impl IntegerCompareOperation {
	handle_sources!(
		(lhs, link),
		(rhs, link),
		(r#type, ignore),
		(operator, ignore)
	);
}

impl IntegerNarrow {
	handle_sources!((source, link));
}

impl IntegerWiden {
	handle_sources!((source, link));
}

impl IntegerExtend {
	handle_sources!((source, link), (r#type, ignore));
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
	handle_sources!((source, link), (r#type, ignore), (operator, ignore));
}

impl NumberBinaryOperation {
	handle_sources!(
		(lhs, link),
		(rhs, link),
		(r#type, ignore),
		(operator, ignore)
	);
}

impl NumberCompareOperation {
	handle_sources!(
		(lhs, link),
		(rhs, link),
		(r#type, ignore),
		(operator, ignore)
	);
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
	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			initializer,
			minimum: _,
			maximum: _,
		} = self;

		for item in initializer {
			handler(item.0.0);
		}
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			initializer,
			minimum: _,
			maximum: _,
		} = self;

		for item in initializer {
			handler(&mut item.0.0);
		}
	}

	fn for_each_argument<H: FnMut(Link)>(&self, mut handler: H) {
		let Self {
			initializer,
			minimum: _,
			maximum: _,
		} = self;

		for item in initializer {
			handler(item.0);
		}
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			initializer,
			minimum: _,
			maximum: _,
		} = self;

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
	handle_sources!((source, method), (r#type, ignore));
}

impl MemoryStore {
	handle_sources!((destination, method), (source, link), (r#type, ignore));
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

impl Node {
	pub fn for_each_requirement<H: FnMut(u32)>(&self, handler: H) {
		match self {
			Self::LambdaOut(node) => node.for_each_requirement(handler),
			Self::RegionIn(node) => node.for_each_requirement(handler),
			Self::RegionOut(node) => node.for_each_requirement(handler),
			Self::GammaOut(node) => node.for_each_requirement(handler),
			Self::ThetaOut(node) => node.for_each_requirement(handler),
			Self::OmegaOut(node) => node.for_each_requirement(handler),

			_ => {}
		}
	}

	pub fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		for_each_visit!(self, for_each_id, handler);
	}

	pub fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		for_each_visit!(self, for_each_mut_id, handler);
	}

	pub fn for_each_argument<H: FnMut(Link)>(&self, mut handler: H) {
		for_each_visit!(self, for_each_argument, handler);
	}

	pub fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		for_each_visit!(self, for_each_mut_argument, handler);
	}
}
