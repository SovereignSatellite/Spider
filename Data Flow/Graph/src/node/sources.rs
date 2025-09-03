use alloc::vec::Vec;

use crate::DataFlowGraph;

use super::{
	item::Node,
	link::Link,
	mvp::{
		Call, DataDrop, DataNew, ElementsDrop, ElementsNew, GlobalGet, GlobalNew, GlobalSet,
		Identity, IntegerBinaryOperation, IntegerCompareOperation, IntegerConvertToNumber,
		IntegerExtend, IntegerNarrow, IntegerTransmuteToNumber, IntegerUnaryOperation,
		IntegerWiden, Location, MemoryCopy, MemoryFill, MemoryGrow, MemoryInit, MemoryLoad,
		MemoryNew, MemorySize, MemoryStore, Merge, NumberBinaryOperation, NumberCompareOperation,
		NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger, NumberUnaryOperation,
		NumberWiden, RefIsNull, TableCopy, TableFill, TableGet, TableGrow, TableInit, TableNew,
		TableSet, TableSize,
	},
	nested::{
		Export, GammaIn, GammaOut, Import, LambdaIn, LambdaOut, OmegaIn, OmegaOut, RegionIn,
		RegionOut, ThetaIn, ThetaOut,
	},
};

macro_rules! for_each_visit {
	($self:ident, $visit:ident, $handler:ident) => {
		match $self {
			Self::LambdaIn(lambda_in) => lambda_in.$visit($handler),
			Self::LambdaOut(lambda_out) => lambda_out.$visit($handler),
			Self::RegionIn(region_in) => region_in.$visit($handler),
			Self::RegionOut(region_out) => region_out.$visit($handler),
			Self::GammaIn(gamma_in) => gamma_in.$visit($handler),
			Self::GammaOut(gamma_out) => gamma_out.$visit($handler),
			Self::ThetaIn(theta_in) => theta_in.$visit($handler),
			Self::ThetaOut(theta_out) => theta_out.$visit($handler),
			Self::OmegaIn(omega_in) => omega_in.$visit($handler),
			Self::OmegaOut(omega_out) => omega_out.$visit($handler),
			Self::Import(import) => import.$visit($handler),
			Self::Host(host) => host.$visit(&mut $handler),
			Self::Trap | Self::Null | Self::I32(_) | Self::I64(_) | Self::F32(_) | Self::F64(_) => {
			}
			Self::Identity(identity) => identity.$visit($handler),
			Self::Call(call) => call.$visit($handler),
			Self::Merge(merge) => merge.$visit($handler),
			Self::RefIsNull(ref_is_null) => ref_is_null.$visit($handler),
			Self::IntegerUnaryOperation(integer_unary_operation) => {
				integer_unary_operation.$visit($handler)
			}
			Self::IntegerBinaryOperation(integer_binary_operation) => {
				integer_binary_operation.$visit($handler)
			}
			Self::IntegerCompareOperation(integer_compare_operation) => {
				integer_compare_operation.$visit($handler)
			}
			Self::IntegerNarrow(integer_narrow) => integer_narrow.$visit($handler),
			Self::IntegerWiden(integer_widen) => integer_widen.$visit($handler),
			Self::IntegerExtend(integer_extend) => integer_extend.$visit($handler),
			Self::IntegerConvertToNumber(integer_convert_to_number) => {
				integer_convert_to_number.$visit($handler)
			}
			Self::IntegerTransmuteToNumber(integer_transmute_to_number) => {
				integer_transmute_to_number.$visit($handler)
			}
			Self::NumberUnaryOperation(number_unary_operation) => {
				number_unary_operation.$visit($handler)
			}
			Self::NumberBinaryOperation(number_binary_operation) => {
				number_binary_operation.$visit($handler)
			}
			Self::NumberCompareOperation(number_compare_operation) => {
				number_compare_operation.$visit($handler)
			}
			Self::NumberNarrow(number_narrow) => number_narrow.$visit($handler),
			Self::NumberWiden(number_widen) => number_widen.$visit($handler),
			Self::NumberTruncateToInteger(number_truncate_to_integer) => {
				number_truncate_to_integer.$visit($handler)
			}
			Self::NumberTransmuteToInteger(number_transmute_to_integer) => {
				number_transmute_to_integer.$visit($handler)
			}
			Self::GlobalNew(global_new) => global_new.$visit($handler),
			Self::GlobalGet(global_get) => global_get.$visit($handler),
			Self::GlobalSet(global_set) => global_set.$visit($handler),
			Self::TableNew(table_new) => table_new.$visit($handler),
			Self::TableGet(table_get) => table_get.$visit($handler),
			Self::TableSet(table_set) => table_set.$visit($handler),
			Self::TableSize(table_size) => table_size.$visit($handler),
			Self::TableGrow(table_grow) => table_grow.$visit($handler),
			Self::TableFill(table_fill) => table_fill.$visit($handler),
			Self::TableCopy(table_copy) => table_copy.$visit($handler),
			Self::TableInit(table_init) => table_init.$visit($handler),
			Self::ElementsNew(elements_new) => elements_new.$visit($handler),
			Self::ElementsDrop(elements_drop) => elements_drop.$visit($handler),
			Self::MemoryNew(memory_new) => memory_new.$visit($handler),
			Self::MemoryLoad(memory_load) => memory_load.$visit($handler),
			Self::MemoryStore(memory_store) => memory_store.$visit($handler),
			Self::MemorySize(memory_size) => memory_size.$visit($handler),
			Self::MemoryGrow(memory_grow) => memory_grow.$visit($handler),
			Self::MemoryFill(memory_fill) => memory_fill.$visit($handler),
			Self::MemoryCopy(memory_copy) => memory_copy.$visit($handler),
			Self::MemoryInit(memory_init) => memory_init.$visit($handler),
			Self::DataNew(data_new) => data_new.$visit($handler),
			Self::DataDrop(data_drop) => data_drop.$visit($handler),
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
			dependencies,
		} = self;

		handler(*output);

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
		let Self { input, results: _ } = self;

		handler(*input);
	}

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self { input, results } = self;

		handler(*input);
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
		let Self { input, output: _ } = self;

		handler(*input);
	}

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self { input, output } = self;

		handler(*input);
		handler(*output);
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
		} = self;

		handler(*input);
	}

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			input,
			output,
			results,
		} = self;

		handler(*input);
		handler(*output);
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
			condition,
			arguments,
		} = self;

		handler(*output);
		handler(condition.0);
		for_each_link_list(arguments, handler);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			output,
			condition,
			arguments,
		} = self;

		handler(output);
		handler(&mut condition.0);
		for_each_mut_link_list(arguments, handler);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, mut handler: H) {
		let Self {
			output: _,
			condition,
			arguments,
		} = self;

		handler(*condition);
		arguments.iter().copied().for_each(handler);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			output: _,
			condition,
			arguments,
		} = self;

		handler(condition);
		arguments.iter_mut().for_each(handler);
	}
}

impl GammaOut {
	#[must_use]
	pub fn ports_output(&self, graph: &DataFlowGraph) -> usize {
		let first = *self.regions.first().unwrap();

		graph.get(first).as_region_out().unwrap().ports_output()
	}

	fn for_each_requirement<H: FnMut(u32)>(&self, mut handler: H) {
		let Self { input, regions } = self;

		handler(*input);
		regions.iter().copied().for_each(handler);
	}

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self { input, regions } = self;

		handler(*input);
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
		let Self { output, arguments } = self;

		handler(*output);
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
			condition: _,
			results: _,
		} = self;

		handler(*input);
	}

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			input,
			condition,
			results,
		} = self;

		handler(*input);
		handler(condition.0);
		for_each_link_list(results, handler);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			input,
			condition,
			results,
		} = self;

		handler(input);
		handler(&mut condition.0);
		for_each_mut_link_list(results, handler);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, mut handler: H) {
		let Self {
			input: _,
			condition,
			results,
		} = self;

		handler(*condition);
		results.iter().copied().for_each(handler);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			input: _,
			condition,
			results,
		} = self;

		handler(condition);
		results.iter_mut().for_each(handler);
	}
}

impl OmegaIn {
	pub const ENVIRONMENT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self { output } = self;

		handler(*output);
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
		} = self;

		handler(*reference);
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
		} = self;

		handler(*input);
	}

	fn for_each_id<H: FnMut(u32)>(&self, mut handler: H) {
		let Self {
			input,
			state,
			exports,
		} = self;

		handler(*input);
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
			exports,
		} = self;

		handler(*state);

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
		} = self;

		handler(*environment);
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

impl Call {
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
			arguments,
			results: _,
			states: _,
		} = self;

		handler(*function);
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

impl Merge {
	fn for_each_id<H: FnMut(u32)>(&self, handler: H) {
		let Self { states } = self;

		for_each_link_list(states, handler);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, handler: H) {
		let Self { states } = self;

		for_each_mut_link_list(states, handler);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, handler: H) {
		let Self { states } = self;

		states.iter().copied().for_each(handler);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, handler: H) {
		let Self { states } = self;

		states.iter_mut().for_each(handler);
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
	fn for_each_id<H: FnMut(u32)>(self, mut handler: H) {
		let Self {
			initializer,
			minimum: _,
			maximum: _,
		} = self;

		handler(initializer.0);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, mut handler: H) {
		let Self {
			initializer,
			minimum: _,
			maximum: _,
		} = self;

		handler(&mut initializer.0);
	}

	fn for_each_argument<H: FnMut(Link)>(self, mut handler: H) {
		let Self {
			initializer,
			minimum: _,
			maximum: _,
		} = self;

		handler(initializer);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self {
			initializer,
			minimum: _,
			maximum: _,
		} = self;

		handler(initializer);
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

impl TableInit {
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

impl ElementsNew {
	fn for_each_id<H: FnMut(u32)>(&self, handler: H) {
		let Self { content } = self;

		for_each_link_list(content, handler);
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&mut self, handler: H) {
		let Self { content } = self;

		for_each_mut_link_list(content, handler);
	}

	fn for_each_argument<H: FnMut(Link)>(&self, handler: H) {
		let Self { content } = self;

		content.iter().copied().for_each(handler);
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&mut self, handler: H) {
		let Self { content } = self;

		content.iter_mut().for_each(handler);
	}
}

impl ElementsDrop {
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
	fn for_each_id<H: FnMut(u32)>(self, _handler: H) {
		let Self {
			minimum: _,
			maximum: _,
		} = self;
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(self, _handler: H) {
		let Self {
			minimum: _,
			maximum: _,
		} = self;
	}

	fn for_each_argument<H: FnMut(Link)>(self, _handler: H) {
		let Self {
			minimum: _,
			maximum: _,
		} = self;
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(self, _handler: H) {
		let Self {
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

impl MemoryInit {
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

impl DataNew {
	fn for_each_id<H: FnMut(u32)>(&self, _handler: H) {
		let Self { content: _ } = self;
	}

	fn for_each_mut_id<H: FnMut(&mut u32)>(&self, _handler: H) {
		let Self { content: _ } = self;
	}

	fn for_each_argument<H: FnMut(Link)>(&self, _handler: H) {
		let Self { content: _ } = self;
	}

	fn for_each_mut_argument<H: FnMut(&mut Link)>(&self, _handler: H) {
		let Self { content: _ } = self;
	}
}

impl DataDrop {
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
			Self::RegionIn(region_in) => region_in.ports_output(graph),
			Self::GammaOut(gamma_out) => gamma_out.ports_output(graph),
			Self::ThetaIn(theta_in) => theta_in.ports_output(),
			Self::ThetaOut(theta_out) => theta_out.ports_output(),

			_ => return None,
		};

		Some(result)
	}

	#[must_use]
	pub const fn as_ports(&self) -> Option<&Vec<Link>> {
		let ports = match self {
			Self::LambdaIn(lambda_in) => &lambda_in.dependencies,
			Self::LambdaOut(lambda_out) => &lambda_out.results,
			Self::RegionOut(region_out) => &region_out.results,
			Self::GammaIn(gamma_in) => &gamma_in.arguments,
			Self::ThetaIn(theta_in) => &theta_in.arguments,
			Self::ThetaOut(theta_out) => &theta_out.results,

			_ => return None,
		};

		Some(ports)
	}

	#[must_use]
	pub const fn as_mut_ports(&mut self) -> Option<&mut Vec<Link>> {
		let ports = match self {
			Self::LambdaIn(lambda_in) => &mut lambda_in.dependencies,
			Self::LambdaOut(lambda_out) => &mut lambda_out.results,
			Self::RegionOut(region_out) => &mut region_out.results,
			Self::GammaIn(gamma_in) => &mut gamma_in.arguments,
			Self::ThetaIn(theta_in) => &mut theta_in.arguments,
			Self::ThetaOut(theta_out) => &mut theta_out.results,

			_ => return None,
		};

		Some(ports)
	}

	pub fn for_each_requirement<H: FnMut(u32)>(&self, handler: H) {
		match self {
			Self::LambdaOut(lambda_out) => lambda_out.for_each_requirement(handler),
			Self::RegionIn(region_in) => region_in.for_each_requirement(handler),
			Self::RegionOut(region_out) => region_out.for_each_requirement(handler),
			Self::GammaOut(gamma_out) => gamma_out.for_each_requirement(handler),
			Self::ThetaOut(theta_out) => theta_out.for_each_requirement(handler),
			Self::OmegaOut(omega_out) => omega_out.for_each_requirement(handler),

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
