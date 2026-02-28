#![no_std]

use alloc::{sync::Arc, vec::Vec};
use ir_graph::{
	DataFlowGraph, Link, Node,
	control::{
		GammaIn, GammaOut, Import, OmegaIn, OmegaOut, RegionIn, RegionOut, ThetaIn, ThetaOut,
	},
	list::resizable::Resizable,
	simple::{
		Apply, Fence, IntegerBinaryOperation, IntegerBinaryOperator, IntegerCompareOperation,
		IntegerCompareOperator, IntegerType, LoadType, Location, MemoryLoad, MemoryNew,
		MemoryStore, StoreType,
	},
};

extern crate alloc;

#[expect(clippy::cast_possible_truncation)]
const CELL_SIZE: u32 = size_of::<u32>() as u32;

const MEMORY_SIZE: u32 = 1_024 * 4 * CELL_SIZE;

struct Block {
	gamma: u32,
	region: u32,
	theta: u32,
}

pub struct TuringMachineLifter {
	loads: Vec<Link>,
	store: Link,
	offset: Link,

	io: Link,
	ask: Link,
	tell: Link,

	blocks: Vec<Block>,
}

impl TuringMachineLifter {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			loads: Vec::new(),
			store: Link::DANGLING,
			offset: Link::DANGLING,

			io: Link::DANGLING,
			ask: Link::DANGLING,
			tell: Link::DANGLING,

			blocks: Vec::new(),
		}
	}

	fn create_memory(&mut self, graph: &mut DataFlowGraph) {
		self.loads.clear();

		self.store = MemoryNew::add_into(graph, Vec::new(), MEMORY_SIZE, MEMORY_SIZE);
		self.offset = Node::add_i32_into(graph, 0);
	}

	fn create_io(&mut self, graph: &mut DataFlowGraph, omega_in: u32) {
		let environment = Link(omega_in, OmegaIn::ENVIRONMENT_PORT);
		let namespace = Arc::<str>::from("turing");

		self.io = Link(omega_in, OmegaIn::STATE_PORT);
		self.ask = Import::add_into(graph, environment, namespace.clone(), "ask".into());
		self.tell = Import::add_into(graph, environment, namespace, "tell".into());
	}

	fn reconcile_store(&mut self, graph: &mut DataFlowGraph) -> Link {
		let sources: Resizable<_, _> = self.loads.iter().copied().collect();

		self.loads.clear();

		match *sources {
			[state] => state,
			[] => self.store,
			[..] => Link(Fence::add_into(graph, sources), 0),
		}
	}

	fn do_load(&mut self, graph: &mut DataFlowGraph) -> Link {
		let source = Location {
			reference: self.store,
			offset: self.offset,
		};
		let (result, state) = MemoryLoad::add_into(graph, source, LoadType::I32);

		self.loads.push(state);

		result
	}

	fn do_store(&mut self, graph: &mut DataFlowGraph, source: Link) {
		let destination = Location {
			reference: self.reconcile_store(graph),
			offset: self.offset,
		};

		self.store = MemoryStore::add_into(graph, destination, source, StoreType::I32);
	}

	fn do_condition(&mut self, graph: &mut DataFlowGraph) -> Link {
		let lhs = self.do_load(graph);
		let rhs = Node::add_i32_into(graph, 0);

		IntegerCompareOperation::add_into(
			graph,
			lhs,
			rhs,
			IntegerType::I32,
			IntegerCompareOperator::NotEqual,
		)
	}

	fn handle_offset_operation(
		&mut self,
		graph: &mut DataFlowGraph,
		operator: IntegerBinaryOperator,
	) {
		let rhs = Node::add_i32_into(graph, CELL_SIZE.try_into().unwrap());

		self.offset =
			IntegerBinaryOperation::add_into(graph, self.offset, rhs, IntegerType::I32, operator);
	}

	fn handle_memory_operation(
		&mut self,
		graph: &mut DataFlowGraph,
		operator: IntegerBinaryOperator,
	) {
		let lhs = self.do_load(graph);
		let rhs = Node::add_i32_into(graph, 1);
		let source = IntegerBinaryOperation::add_into(graph, lhs, rhs, IntegerType::I32, operator);

		self.do_store(graph, source);
	}

	fn handle_ask(&mut self, graph: &mut DataFlowGraph) {
		let apply = Apply::add_into(graph, self.ask, alloc::vec![self.io], 2);

		self.io = Link(apply, 0);

		self.do_store(graph, Link(apply, 1));
	}

	fn handle_tell(&mut self, graph: &mut DataFlowGraph) {
		let source = self.do_load(graph);
		let apply = Apply::add_into(graph, self.tell, alloc::vec![source, self.io], 1);

		self.io = Link(apply, 0);
	}

	fn pull_all_active(&mut self, graph: &mut DataFlowGraph) -> Vec<Link> {
		alloc::vec![
			self.reconcile_store(graph),
			self.offset,
			self.io,
			self.ask,
			self.tell,
		]
	}

	const fn push_all_active(&mut self, source: u32) {
		self.store = Link(source, 0);
		self.offset = Link(source, 1);
		self.io = Link(source, 3);
		self.ask = Link(source, 2);
		self.tell = Link(source, 4);
	}

	fn create_if_entry(&mut self, graph: &mut DataFlowGraph) -> u32 {
		let condition = self.do_condition(graph);
		let arguments = self.pull_all_active(graph);

		GammaIn::add_into(graph, arguments, condition)
	}

	fn create_true_entry(&mut self, graph: &mut DataFlowGraph, gamma_in: u32) -> u32 {
		let region_in = RegionIn::add_into(graph, gamma_in);

		self.push_all_active(region_in);

		region_in
	}

	fn create_repeat_entry(&mut self, graph: &mut DataFlowGraph) -> u32 {
		let arguments = self.pull_all_active(graph);
		let theta_in = ThetaIn::add_into(graph, arguments);

		self.push_all_active(theta_in);

		theta_in
	}

	fn handle_block_start(&mut self, graph: &mut DataFlowGraph) {
		let gamma = self.create_if_entry(graph);
		let region = self.create_true_entry(graph, gamma);
		let theta = self.create_repeat_entry(graph);

		self.blocks.push(Block {
			gamma,
			region,
			theta,
		});
	}

	fn create_repeat_exit(&mut self, graph: &mut DataFlowGraph, theta_in: u32) {
		let condition = self.do_condition(graph);
		let results = self.pull_all_active(graph);
		let theta_out = ThetaOut::add_into(graph, theta_in, results, condition);

		self.push_all_active(theta_out);
	}

	fn create_true_exit(&mut self, graph: &mut DataFlowGraph, region_in: u32) -> u32 {
		let results = self.pull_all_active(graph);

		RegionOut::add_into(graph, region_in, results)
	}

	fn create_false_case(&mut self, graph: &mut DataFlowGraph, gamma_in: u32) -> u32 {
		RegionOut::add_scoped_into(graph, gamma_in, |graph, region_in| {
			self.push_all_active(region_in);
			self.pull_all_active(graph)
		})
	}

	fn create_if_exit(&mut self, graph: &mut DataFlowGraph, region_in: u32, gamma_in: u32) {
		let on_true = self.create_true_exit(graph, region_in);
		let on_false = self.create_false_case(graph, gamma_in);
		let gamma_out = GammaOut::add_into(graph, gamma_in, alloc::vec![on_false, on_true]);

		self.push_all_active(gamma_out);
	}

	fn handle_block_end(&mut self, graph: &mut DataFlowGraph) {
		let Block {
			gamma,
			region,
			theta,
		} = self.blocks.pop().unwrap();

		self.create_repeat_exit(graph, theta);
		self.create_if_exit(graph, region, gamma);
	}

	fn handle_code(&mut self, graph: &mut DataFlowGraph, source: &str) {
		self.blocks.clear();

		for character in source.chars() {
			match character {
				'>' => self.handle_offset_operation(graph, IntegerBinaryOperator::Add),
				'<' => self.handle_offset_operation(graph, IntegerBinaryOperator::Subtract),

				'+' => self.handle_memory_operation(graph, IntegerBinaryOperator::Add),
				'-' => self.handle_memory_operation(graph, IntegerBinaryOperator::Subtract),

				',' => self.handle_ask(graph),
				'.' => self.handle_tell(graph),

				'[' => self.handle_block_start(graph),
				']' => self.handle_block_end(graph),

				_ => {}
			}
		}
	}

	pub fn run(&mut self, graph: &mut DataFlowGraph, source: &str) -> u32 {
		let omega_in = OmegaIn::add_into(graph);

		self.create_memory(graph);
		self.create_io(graph, omega_in);
		self.handle_code(graph, source);

		OmegaOut::add_into(graph, omega_in, self.io, Vec::new())
	}
}

impl Default for TuringMachineLifter {
	fn default() -> Self {
		Self::new()
	}
}
