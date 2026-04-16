//! WebAssembly control flow builder for converting operators into structured IR.

#![no_std]

extern crate alloc;

use wasmparser::{BlockType, OperatorsReader};

use web_assembly_graph::ControlFlowGraph;
use web_assembly_structurer::ControlFlowStructurer;

use self::{expression_builder::ExpressionBuilder, post_order_sorter::PostOrderSorter};

pub use self::types::Types;

mod code_builder;
mod expression_builder;
mod post_order_sorter;
mod stack_builder;
mod types;

/// Builds a structured control flow graph from WebAssembly operators.
pub struct ControlFlowBuilder {
	expression_builder: ExpressionBuilder,
	post_order_sorter: PostOrderSorter,
	control_flow_structurer: ControlFlowStructurer,
}

impl ControlFlowBuilder {
	#[must_use]
	/// Creates a new control flow builder.
	pub const fn new() -> Self {
		Self {
			expression_builder: ExpressionBuilder::new(),
			post_order_sorter: PostOrderSorter::new(),
			control_flow_structurer: ControlFlowStructurer::new(),
		}
	}

	/// Builds and restructures the control flow graph from WebAssembly operators.
	#[expect(
		clippy::too_many_arguments,
		reason = "entry point requires all builder state"
	)]
	pub fn run(
		&mut self,
		graph: &mut ControlFlowGraph,
		types: &Types,
		function_type: BlockType,
		locals: u16,
		operators: OperatorsReader<'_>,
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
