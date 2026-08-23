use wasmparser::{BlockType, OperatorsReader};

use web_assembly_control_flow::{
	ControlFlowGraph, ControlFlowStructurer, TopologicalOrderer, instruction::ReservedLocal,
};

use self::operator_decoder::OperatorDecoder;
use crate::module::TypeRegistry;

mod block_builder;
mod operand_stack;
mod operator_decoder;

const DECODER_SCRATCH: u16 = ReservedLocal::DecoderScratch as u16;
const LOCAL_BASE: u16 = ReservedLocal::COUNT;

pub struct FunctionDecoder {
	operator_decoder: OperatorDecoder,
	topological_orderer: TopologicalOrderer,
	structurer: ControlFlowStructurer,
}

impl FunctionDecoder {
	pub const fn new() -> Self {
		Self {
			operator_decoder: OperatorDecoder::new(),
			topological_orderer: TopologicalOrderer::new(),
			structurer: ControlFlowStructurer::new(),
		}
	}

	pub const fn graph(&self) -> &ControlFlowGraph {
		self.operator_decoder.graph()
	}

	pub const fn graph_mut(&mut self) -> &mut ControlFlowGraph {
		self.operator_decoder.graph_mut()
	}

	pub fn run(
		&mut self,
		types: &TypeRegistry,
		function_type: BlockType,
		locals: u16,
		operators: OperatorsReader<'_>,
	) {
		self.operator_decoder
			.run(types, function_type, locals, operators);

		let graph = self.operator_decoder.graph_mut();

		self.topological_orderer.run(&mut graph.basic_blocks, 0);

		let exit = graph.add_no_operation();

		self.structurer.run(graph, 0, exit);
		self.topological_orderer.run(&mut graph.basic_blocks, 0);
	}
}
