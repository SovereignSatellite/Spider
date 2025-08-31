#![no_std]

extern crate alloc;

mod basic_block_builder;
mod code_builder;
mod sorter;
mod stack_builder;
mod types;

use control_flow_graph::ControlFlowGraph;
use control_flow_structurer::Structurer;
use wasmparser::{BlockType, OperatorsReader};

use self::{basic_block_builder::BasicBlockBuilder, sorter::Sorter};

pub use self::types::Types;

pub struct ControlFlowBuilder {
	basic_block_builder: BasicBlockBuilder,
	structurer: Structurer,
	sorter: Sorter,
}

impl ControlFlowBuilder {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			basic_block_builder: BasicBlockBuilder::new(),
			structurer: Structurer::new(),
			sorter: Sorter::new(),
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
		self.basic_block_builder
			.run(graph, types, function_type, locals, operators);

		self.sorter.run(&mut graph.basic_blocks, 0);

		let exit = graph.add_no_operation();

		self.structurer.run(graph, 0, exit);
		self.sorter.run(&mut graph.basic_blocks, 0);
	}
}

impl Default for ControlFlowBuilder {
	fn default() -> Self {
		Self::new()
	}
}
