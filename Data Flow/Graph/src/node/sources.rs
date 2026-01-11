use alloc::vec::Vec;

use crate::DataFlowGraph;

use super::{
	Link, Node,
	base::{
		Apply, GlobalGet, GlobalNew, GlobalSet, Identity, IntegerBinaryOperation,
		IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend, IntegerNarrow,
		IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden, Location, MemoryCopy,
		MemoryDrop, MemoryFill, MemoryGrow, MemoryLoad, MemoryNew, MemorySize, MemoryStore, Merge,
		NumberBinaryOperation, NumberCompareOperation, NumberNarrow, NumberTransmuteToInteger,
		NumberTruncateToInteger, NumberUnaryOperation, NumberWiden, RefIsNull, TableCopy,
		TableDrop, TableFill, TableGet, TableGrow, TableNew, TableSet, TableSize,
	},
	control::{
		Export, GammaIn, GammaOut, Import, LambdaIn, LambdaOut, OmegaIn, OmegaOut, RegionIn,
		RegionOut, ThetaIn, ThetaOut,
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
			Self::Merge(node) => node.$visit($handler),
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

fn for_each_link_list<H: FnMut(u32)>(list: &[Link], handler: H) {
	list.iter().map(|link| link.0).for_each(handler);
}

fn for_each_mut_link_list<H: FnMut(&mut u32)>(list: &mut [Link], handler: H) {
	list.iter_mut().map(|link| &mut link.0).for_each(handler);
}

impl LambdaIn {
	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			output,
			r#type: _,
			ref dependencies,
		} = *self;

		handler(output);

		for_each_link_list(dependencies, handler);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			output,
			r#type: _,
			dependencies,
		} = self;

		handler(output);

		for_each_mut_link_list(dependencies, handler);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, handler: H) {
		let Self {
			output: _,
			r#type: _,
			dependencies,
		} = self;

		dependencies.iter().copied().for_each(handler);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, handler: H) {
		let Self {
			output: _,
			r#type: _,
			dependencies,
		} = self;

		dependencies.iter_mut().for_each(handler);
	}
}

impl LambdaOut {
	fn for_each_requirement<H: FnMut(u32)>(&self, mut handler: H) {
		let Self { input, results: _ } = *self;

		handler(input);
	}

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self { input, ref results } = *self;

		handler(input);
		for_each_link_list(results, handler);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { input, results } = self;

		handler(input);
		for_each_mut_link_list(results, handler);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, handler: H) {
		let Self { input: _, results } = self;

		results.iter().copied().for_each(handler);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, handler: H) {
		let Self { input: _, results } = self;

		results.iter_mut().for_each(handler);
	}
}

impl RegionIn {
	fn ports_output(&self, graph: &DataFlowGraph) -> usize {
		graph.get(self.input).as_gamma_in().unwrap().ports_output()
	}

	fn for_each_requirement<H: FnMut(u32)>(&self, mut handler: H) {
		let Self { input, output: _ } = *self;

		handler(input);
	}

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self { input, output } = *self;

		handler(input);
		handler(output);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { input, output } = self;

		handler(input);
		handler(output);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, _handler: H) {
		let Self {
			input: _,
			output: _,
		} = self;
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&self, _handler: H) {
		let Self {
			input: _,
			output: _,
		} = self;
	}
}

impl RegionOut {
	const fn ports_output(&self) -> usize {
		self.results.len()
	}

	fn for_each_requirement<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			input,
			output: _,
			results: _,
		} = *self;

		handler(input);
	}

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			input,
			output,
			ref results,
		} = *self;

		handler(input);
		handler(output);
		for_each_link_list(results, handler);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			input,
			output,
			results,
		} = self;

		handler(input);
		handler(output);
		for_each_mut_link_list(results, handler);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, handler: H) {
		let Self {
			input: _,
			output: _,
			results,
		} = self;

		results.iter().copied().for_each(handler);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, handler: H) {
		let Self {
			input: _,
			output: _,
			results,
		} = self;

		results.iter_mut().for_each(handler);
	}
}

impl GammaIn {
	const fn ports_output(&self) -> usize {
		self.arguments.len()
	}

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			output,
			ref arguments,
			condition,
		} = *self;

		handler(output);
		for_each_link_list(arguments, &mut handler);

		handler(condition.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			output,
			arguments,
			condition,
		} = self;

		handler(output);
		for_each_mut_link_list(arguments, &mut handler);

		handler(&mut condition.0);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, mut handler: H) {
		let Self {
			output: _,
			ref arguments,
			condition,
		} = *self;

		arguments.iter().copied().for_each(&mut handler);

		handler(condition);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			output: _,
			arguments,
			condition,
		} = self;

		arguments.iter_mut().for_each(&mut handler);

		handler(condition);
	}
}

impl GammaOut {
	#[must_use]
	pub fn ports_output(&self, graph: &DataFlowGraph) -> usize {
		let first = *self.regions.first().unwrap();

		graph.get(first).as_region_out().unwrap().ports_output()
	}

	fn for_each_requirement<H: FnMut(u32)>(&self, mut handler: H) {
		let Self { input, ref regions } = *self;

		handler(input);
		regions.iter().copied().for_each(handler);
	}

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self { input, ref regions } = *self;

		handler(input);
		regions.iter().copied().for_each(handler);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { input, regions } = self;

		handler(input);
		regions.iter_mut().for_each(handler);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, _handler: H) {
		let Self {
			input: _,
			regions: _,
		} = self;
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&self, _handler: H) {
		let Self {
			input: _,
			regions: _,
		} = self;
	}
}

impl ThetaIn {
	const fn ports_output(&self) -> usize {
		self.arguments.len()
	}

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			output,
			ref arguments,
		} = *self;

		handler(output);
		for_each_link_list(arguments, handler);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { output, arguments } = self;

		handler(output);
		for_each_mut_link_list(arguments, handler);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, handler: H) {
		let Self {
			output: _,
			arguments,
		} = self;

		arguments.iter().copied().for_each(handler);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, handler: H) {
		let Self {
			output: _,
			arguments,
		} = self;

		arguments.iter_mut().for_each(handler);
	}
}

impl ThetaOut {
	const fn ports_output(&self) -> usize {
		self.results.len()
	}

	fn for_each_requirement<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			input,
			results: _,
			condition: _,
		} = *self;

		handler(input);
	}

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			input,
			ref results,
			condition,
		} = *self;

		handler(input);
		for_each_link_list(results, &mut handler);

		handler(condition.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			input,
			results,
			condition,
		} = self;

		handler(input);
		for_each_mut_link_list(results, &mut handler);

		handler(&mut condition.0);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, mut handler: H) {
		let Self {
			input: _,
			ref results,
			condition,
		} = *self;

		results.iter().copied().for_each(&mut handler);

		handler(condition);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			input: _,
			results,
			condition,
		} = self;

		results.iter_mut().for_each(&mut handler);

		handler(condition);
	}
}

impl OmegaIn {
	pub const ENVIRONMENT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self { output } = *self;

		handler(output);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { output } = self;

		handler(output);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, _handler: H) {
		let Self { output: _ } = self;
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&self, _handler: H) {
		let Self { output: _ } = self;
	}
}

impl Export {
	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			identifier: _,
			reference,
		} = self;

		handler(reference.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			identifier: _,
			reference,
		} = self;

		handler(&mut reference.0);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, mut handler: H) {
		let Self {
			identifier: _,
			reference,
		} = *self;

		handler(reference);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			identifier: _,
			reference,
		} = self;

		handler(reference);
	}
}

impl OmegaOut {
	fn for_each_requirement<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			input,
			state: _,
			exports: _,
		} = *self;

		handler(input);
	}

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			input,
			state,
			ref exports,
		} = *self;

		handler(input);
		handler(state.0);

		for export in exports {
			export.for_each_id(&mut handler);
		}
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			input,
			state,
			exports,
		} = self;

		handler(input);
		handler(&mut state.0);

		for export in exports {
			export.for_each_mut_id(&mut handler);
		}
	}

	fn for_each_argument<H: FnMut(Link)>(&self, mut handler: H) {
		let Self {
			input: _,
			state,
			ref exports,
		} = *self;

		handler(state);

		for export in exports {
			export.for_each_argument(&mut handler);
		}
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			input: _,
			state,
			exports,
		} = self;

		handler(state);

		for export in exports {
			export.for_each_mut_argument(&mut handler);
		}
	}
}

impl Import {
	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			environment,
			namespace: _,
			identifier: _,
		} = self;

		handler(environment.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			environment,
			namespace: _,
			identifier: _,
		} = self;

		handler(&mut environment.0);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, mut handler: H) {
		let Self {
			environment,
			namespace: _,
			identifier: _,
		} = *self;

		handler(environment);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			environment,
			namespace: _,
			identifier: _,
		} = self;

		handler(environment);
	}
}

impl Identity {
	fn for_each_id<H: FnMut(u32)>(&self, handler: H) {
		let Self { sources } = self;

		for_each_link_list(sources, handler);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, handler: H) {
		let Self { sources } = self;

		for_each_mut_link_list(sources, handler);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, handler: H) {
		let Self { sources } = self;

		sources.iter().copied().for_each(handler);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, handler: H) {
		let Self { sources } = self;

		sources.iter_mut().for_each(handler);
	}
}

impl Merge {
	fn for_each_id<H: FnMut(u32)>(&self, handler: H) {
		let Self { sources } = self;

		for_each_link_list(sources, handler);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, handler: H) {
		let Self { sources } = self;

		for_each_mut_link_list(sources, handler);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, handler: H) {
		let Self { sources } = self;

		sources.iter().copied().for_each(handler);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, handler: H) {
		let Self { sources } = self;

		sources.iter_mut().for_each(handler);
	}
}

impl Apply {
	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			function,
			arguments,
			results: _,
			states: _,
		} = self;

		handler(function.0);
		for_each_link_list(arguments, handler);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			function,
			arguments,
			results: _,
			states: _,
		} = self;

		handler(&mut function.0);
		for_each_mut_link_list(arguments, handler);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, mut handler: H) {
		let Self {
			function,
			ref arguments,
			results: _,
			states: _,
		} = *self;

		handler(function);
		arguments.iter().copied().for_each(handler);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			function,
			arguments,
			results: _,
			states: _,
		} = self;

		handler(function);
		arguments.iter_mut().for_each(handler);
	}
}

impl RefIsNull {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}
}

impl IntegerUnaryOperation {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			source,
			r#type: _,
			operator: _,
		} = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			source,
			r#type: _,
			operator: _,
		} = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			source,
			r#type: _,
			operator: _,
		} = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			source,
			r#type: _,
			operator: _,
		} = self;

		handler(source);
	}
}

impl IntegerBinaryOperation {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(lhs.0);
		handler(rhs.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(&mut lhs.0);
		handler(&mut rhs.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(lhs);
		handler(rhs);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(lhs);
		handler(rhs);
	}
}

impl IntegerCompareOperation {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(lhs.0);
		handler(rhs.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(&mut lhs.0);
		handler(&mut rhs.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(lhs);
		handler(rhs);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(lhs);
		handler(rhs);
	}
}

impl IntegerNarrow {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}
}

impl IntegerWiden {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}
}

impl IntegerExtend {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { source, r#type: _ } = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { source, r#type: _ } = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { source, r#type: _ } = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { source, r#type: _ } = self;

		handler(source);
	}
}

impl IntegerConvertToNumber {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			source,
			signed: _,
			to: _,
			from: _,
		} = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			source,
			signed: _,
			to: _,
			from: _,
		} = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			source,
			signed: _,
			to: _,
			from: _,
		} = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			source,
			signed: _,
			to: _,
			from: _,
		} = self;

		handler(source);
	}
}

impl IntegerTransmuteToNumber {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { source, from: _ } = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { source, from: _ } = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { source, from: _ } = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { source, from: _ } = self;

		handler(source);
	}
}

impl NumberUnaryOperation {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			source,
			r#type: _,
			operator: _,
		} = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			source,
			r#type: _,
			operator: _,
		} = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			source,
			r#type: _,
			operator: _,
		} = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			source,
			r#type: _,
			operator: _,
		} = self;

		handler(source);
	}
}

impl NumberBinaryOperation {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(lhs.0);
		handler(rhs.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(&mut lhs.0);
		handler(&mut rhs.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(lhs);
		handler(rhs);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(lhs);
		handler(rhs);
	}
}

impl NumberCompareOperation {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(lhs.0);
		handler(rhs.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(&mut lhs.0);
		handler(&mut rhs.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(lhs);
		handler(rhs);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		handler(lhs);
		handler(rhs);
	}
}

impl NumberTruncateToInteger {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			source,
			signed: _,
			saturate: _,
			to: _,
			from: _,
		} = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			source,
			signed: _,
			saturate: _,
			to: _,
			from: _,
		} = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			source,
			signed: _,
			saturate: _,
			to: _,
			from: _,
		} = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			source,
			signed: _,
			saturate: _,
			to: _,
			from: _,
		} = self;

		handler(source);
	}
}

impl NumberTransmuteToInteger {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { source, from: _ } = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { source, from: _ } = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { source, from: _ } = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { source, from: _ } = self;

		handler(source);
	}
}

impl NumberNarrow {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}
}

impl NumberWiden {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}
}

impl Location {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { reference, offset } = self;

		handler(reference.0);
		handler(offset.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { reference, offset } = self;

		handler(&mut reference.0);
		handler(&mut offset.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { reference, offset } = self;

		handler(reference);
		handler(offset);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { reference, offset } = self;

		handler(reference);
		handler(offset);
	}
}

impl GlobalNew {
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { initializer } = self;

		handler(initializer.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { initializer } = self;

		handler(&mut initializer.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { initializer } = self;

		handler(initializer);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { initializer } = self;

		handler(initializer);
	}
}

impl GlobalGet {
	pub const RESULT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}
}

impl GlobalSet {
	pub const STATE_PORT: u16 = 0;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			destination,
			source,
		} = self;

		handler(destination.0);
		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			destination,
			source,
		} = self;

		handler(&mut destination.0);
		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			destination,
			source,
		} = self;

		handler(destination);
		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			destination,
			source,
		} = self;

		handler(destination);
		handler(source);
	}
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
	pub const RESULT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	fn for_each_id<H: FnMut(u32)>(self, handler: H) {
		let Self { source } = self;

		source.for_each_id(handler);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, handler: H) {
		let Self { source } = self;

		source.for_each_mut_id(handler);
	}

	fn for_each_argument<H: FnMut(Link)>(self, handler: H) {
		let Self { source } = self;

		source.for_each_argument(handler);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, handler: H) {
		let Self { source } = self;

		source.for_each_mut_argument(handler);
	}
}

impl TableSet {
	pub const STATE_PORT: u16 = 0;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			destination,
			source,
		} = self;

		destination.for_each_id(&mut handler);
		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			destination,
			source,
		} = self;

		destination.for_each_mut_id(&mut handler);
		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			destination,
			source,
		} = self;

		destination.for_each_argument(&mut handler);
		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			destination,
			source,
		} = self;

		destination.for_each_mut_argument(&mut handler);
		handler(source);
	}
}

impl TableSize {
	pub const RESULT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}
}

impl TableGrow {
	pub const RESULT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			destination,
			initializer,
			size,
		} = self;

		handler(destination.0);
		handler(initializer.0);
		handler(size.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			destination,
			initializer,
			size,
		} = self;

		handler(&mut destination.0);
		handler(&mut initializer.0);
		handler(&mut size.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			destination,
			initializer,
			size,
		} = self;

		handler(destination);
		handler(initializer);
		handler(size);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			destination,
			initializer,
			size,
		} = self;

		handler(destination);
		handler(initializer);
		handler(size);
	}
}

impl TableFill {
	pub const STATE_PORT: u16 = 0;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.for_each_id(&mut handler);
		handler(source.0);
		handler(size.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.for_each_mut_id(&mut handler);
		handler(&mut source.0);
		handler(&mut size.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.for_each_argument(&mut handler);
		handler(source);
		handler(size);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.for_each_mut_argument(&mut handler);
		handler(source);
		handler(size);
	}
}

impl TableCopy {
	pub const DESTINATION_STATE_PORT: u16 = 0;
	pub const SOURCE_STATE_PORT: u16 = 1;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.for_each_id(&mut handler);
		source.for_each_id(&mut handler);
		handler(size.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.for_each_mut_id(&mut handler);
		source.for_each_mut_id(&mut handler);
		handler(&mut size.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.for_each_argument(&mut handler);
		source.for_each_argument(&mut handler);
		handler(size);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.for_each_mut_argument(&mut handler);
		source.for_each_mut_argument(&mut handler);
		handler(size);
	}
}

impl TableDrop {
	pub const STATE_PORT: u16 = 0;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}
}

impl MemoryNew {
	fn for_each_id<H: FnMut(u32)>(&self, _handler: H) {
		let Self {
			initializer: _,
			minimum: _,
			maximum: _,
		} = self;
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&self, _handler: H) {
		let Self {
			initializer: _,
			minimum: _,
			maximum: _,
		} = self;
	}

	fn for_each_argument<H: FnMut(Link)>(&self, _handler: H) {
		let Self {
			initializer: _,
			minimum: _,
			maximum: _,
		} = self;
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&self, _handler: H) {
		let Self {
			initializer: _,
			minimum: _,
			maximum: _,
		} = self;
	}
}

impl MemoryLoad {
	pub const RESULT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { source, r#type: _ } = self;

		source.for_each_id(&mut handler);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { source, r#type: _ } = self;

		source.for_each_mut_id(&mut handler);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { source, r#type: _ } = self;

		source.for_each_argument(&mut handler);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { source, r#type: _ } = self;

		source.for_each_mut_argument(&mut handler);
	}
}

impl MemoryStore {
	pub const STATE_PORT: u16 = 0;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			destination,
			source,
			r#type: _,
		} = self;

		destination.for_each_id(&mut handler);
		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			destination,
			source,
			r#type: _,
		} = self;

		destination.for_each_mut_id(&mut handler);
		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			destination,
			source,
			r#type: _,
		} = self;

		destination.for_each_argument(&mut handler);
		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			destination,
			source,
			r#type: _,
		} = self;

		destination.for_each_mut_argument(&mut handler);
		handler(source);
	}
}

impl MemorySize {
	pub const RESULT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}
}

impl MemoryGrow {
	pub const RESULT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { destination, size } = self;

		handler(destination.0);
		handler(size.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { destination, size } = self;

		handler(&mut destination.0);
		handler(&mut size.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { destination, size } = self;

		handler(destination);
		handler(size);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { destination, size } = self;

		handler(destination);
		handler(size);
	}
}

impl MemoryFill {
	pub const STATE_PORT: u16 = 0;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			destination,
			byte,
			size,
		} = self;

		destination.for_each_id(&mut handler);
		handler(byte.0);
		handler(size.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			destination,
			byte,
			size,
		} = self;

		destination.for_each_mut_id(&mut handler);
		handler(&mut byte.0);
		handler(&mut size.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			destination,
			byte,
			size,
		} = self;

		destination.for_each_argument(&mut handler);
		handler(byte);
		handler(size);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			destination,
			byte,
			size,
		} = self;

		destination.for_each_mut_argument(&mut handler);
		handler(byte);
		handler(size);
	}
}

impl MemoryCopy {
	pub const DESTINATION_STATE_PORT: u16 = 0;
	pub const SOURCE_STATE_PORT: u16 = 1;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.for_each_id(&mut handler);
		source.for_each_id(&mut handler);
		handler(size.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.for_each_mut_id(&mut handler);
		source.for_each_mut_id(&mut handler);
		handler(&mut size.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.for_each_argument(&mut handler);
		source.for_each_argument(&mut handler);
		handler(size);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.for_each_mut_argument(&mut handler);
		source.for_each_mut_argument(&mut handler);
		handler(size);
	}
}

impl MemoryDrop {
	pub const STATE_PORT: u16 = 0;

	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(&mut source.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { source } = self;

		handler(source);
	}
}

impl Node {
	#[must_use]
	pub fn ports_output(&self, graph: &DataFlowGraph) -> Option<usize> {
		let result = match self {
			Self::RegionIn(node) => node.ports_output(graph),
			Self::GammaOut(node) => node.ports_output(graph),
			Self::ThetaIn(node) => node.ports_output(),
			Self::ThetaOut(node) => node.ports_output(),

			_ => return None,
		};

		Some(result)
	}

	#[must_use]
	pub const fn as_ports(&self) -> Option<&Vec<Link>> {
		let ports = match self {
			Self::LambdaIn(node) => &node.dependencies,
			Self::LambdaOut(node) => &node.results,
			Self::RegionOut(node) => &node.results,
			Self::GammaIn(node) => &node.arguments,
			Self::ThetaIn(node) => &node.arguments,
			Self::ThetaOut(node) => &node.results,

			_ => return None,
		};

		Some(ports)
	}

	#[must_use]
	pub const fn as_mut_ports(&mut self) -> Option<&mut Vec<Link>> {
		let ports = match self {
			Self::LambdaIn(node) => &mut node.dependencies,
			Self::LambdaOut(node) => &mut node.results,
			Self::RegionOut(node) => &mut node.results,
			Self::GammaIn(node) => &mut node.arguments,
			Self::ThetaIn(node) => &mut node.arguments,
			Self::ThetaOut(node) => &mut node.results,

			_ => return None,
		};

		Some(ports)
	}

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
