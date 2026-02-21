#![no_std]
#![expect(clippy::missing_panics_doc)]

extern crate alloc;

mod code_builder;
mod expression_builder;
mod post_order_sorter;
mod stack_builder;
mod types;

use wasmparser::{BlockType, OperatorsReader};
use web_assembly_graph::ControlFlowGraph;
use web_assembly_structurer::ControlFlowStructurer;

use self::{expression_builder::ExpressionBuilder, post_order_sorter::PostOrderSorter};

pub use self::types::Types;

pub struct ControlFlowBuilder {
	expression_builder: ExpressionBuilder,
	post_order_sorter: PostOrderSorter,
	control_flow_structurer: ControlFlowStructurer,
}

impl ControlFlowBuilder {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			expression_builder: ExpressionBuilder::new(),
			post_order_sorter: PostOrderSorter::new(),
			control_flow_structurer: ControlFlowStructurer::new(),
		}
	}

	pub fn run(
		&mut self,
		graph: &mut ControlFlowGraph,
		types: &Types,
		function_type: BlockType,
		locals: u16,
		operators: OperatorsReader,
	) {
		self.expression_builder
			.run(graph, types, function_type, locals, operators);

		self.post_order_sorter.run(&mut graph.basic_blocks, 0);

		let exit = graph.add_no_operation();

		self.control_flow_structurer.run(graph, 0, exit);
		self.post_order_sorter.run(&mut graph.basic_blocks, 0);
	}
}

impl Default for ControlFlowBuilder {
	fn default() -> Self {
		Self::new()
	}
}
