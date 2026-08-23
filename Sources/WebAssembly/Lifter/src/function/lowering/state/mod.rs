#![expect(
	clippy::multiple_inherent_impl,
	reason = "instruction-family handlers are split into focused lowering modules"
)]

use core::iter;

use list::resizable::Resizable;

use ir_graph::{Link, Node, operation};
use web_assembly_control_flow::instruction::{
	Call, F32Constant, F64Constant, GlobalGet, GlobalSet, I32Constant, I64Constant, Instruction,
	IntegerBinaryOperation, LocalBranch, LocalSet, Location, NumberTruncateToInteger, RefFunction,
	RefIsNull, RefNull, Reference, ReferenceType, ReservedLocal,
};

use self::reference_states::ReferenceStates;
use super::{FunctionFrame, LocalKind};

mod memory;
mod numeric;
mod reference_states;
mod table;

const LOCAL_BASE: usize = ReservedLocal::COUNT as usize;

fn to_port(index: usize) -> u16 {
	let Ok(port) = index.try_into() else {
		unreachable!()
	};

	port
}

pub struct LoweringState {
	locals: Vec<Link>,

	trap: Link,
	dependencies: ReferenceStates,
}

impl LoweringState {
	pub const fn new() -> Self {
		Self {
			locals: Vec::new(),

			trap: Link::DANGLING,
			dependencies: ReferenceStates::new(),
		}
	}

	pub fn collect_references(&mut self, instructions: &[Instruction]) {
		self.dependencies.collect_references(instructions);
	}

	pub fn references(&self) -> impl ExactSizeIterator<Item = Reference> + '_ {
		self.dependencies.references()
	}

	fn load_location(&self, kind: ReferenceType, location: Location) -> operation::Location {
		let Location { reference, offset } = location;

		operation::Location {
			reference: self.dependencies.get(kind, reference),
			offset: self.locals[usize::from(offset)],
		}
	}

	fn create_fence(&mut self, nodes: &mut Vec<Node>) {
		let mut sources = vec![self.trap];

		self.dependencies.get_mutable_into(&mut sources);

		let source_count = to_port(sources.len());
		let fence = operation::Fence::add_into(nodes, Resizable::Heap(sources));

		self.trap = Link(fence, 0);

		self.dependencies
			.set_mutable_from((1..source_count).map(|port| Link(fence, port)));
	}

	pub fn capture_outputs(&mut self, nodes: &mut Vec<Node>, result_count: usize) -> Vec<Link> {
		let mut results = Vec::with_capacity(result_count + 1);

		results.extend_from_slice(&self.locals[LOCAL_BASE..LOCAL_BASE + result_count]);

		self.create_fence(nodes);

		results.push(self.trap);

		results
	}

	fn seed_dependencies_from_closure(&mut self, nodes: &mut Vec<Node>, closure: Link) {
		let Ok(count) = u32::try_from(self.dependencies.count()) else {
			unreachable!()
		};
		let extracts = (1..=count).map(|port| operation::Extract::add_into(nodes, closure, port));

		self.dependencies.set_all_from(extracts);
	}

	fn seed_declared_locals(&mut self, nodes: &mut Vec<Node>, kinds: &[LocalKind]) {
		self.locals.extend(kinds.iter().map(|&local| match local {
			LocalKind::I32 => Node::add_i32_into(nodes, 0),
			LocalKind::I64 => Node::add_i64_into(nodes, 0),
			LocalKind::F32 => Node::add_f32_into(nodes, 0.0),
			LocalKind::F64 => Node::add_f64_into(nodes, 0.0),
			LocalKind::Reference => Node::add_null_into(nodes),
		}));
	}

	// The slot layout is [scratch | parameters | declared locals | stack]; the
	// trap rides the argument port directly after the parameters.
	pub fn seed(&mut self, nodes: &mut Vec<Node>, frame: &FunctionFrame<'_>) {
		let &FunctionFrame {
			arguments,
			argument_count,
			local_kinds,
			slot_count,
			..
		} = frame;

		self.seed_dependencies_from_closure(nodes, Link(arguments, 0));

		let null = Node::add_null_into(nodes);
		let zero = Node::add_i32_into(nodes, 0);
		let parameters = (1..=to_port(argument_count)).map(|port| Link(arguments, port));

		self.locals.clear();
		self.locals.extend(iter::repeat_n(zero, LOCAL_BASE));
		self.locals.extend(parameters);
		self.seed_declared_locals(nodes, local_kinds);

		let padding = usize::from(slot_count).saturating_sub(self.locals.len());

		self.locals.extend(iter::repeat_n(null, padding));

		self.trap = Link(arguments, to_port(argument_count + 1));
	}

	pub fn capture_bindings(&self, live: &[u16]) -> Vec<Link> {
		let mut links = Vec::with_capacity(self.dependencies.count() + live.len() + 1);

		self.dependencies.get_all_into(&mut links);

		links.extend(live.iter().map(|&slot| self.locals[usize::from(slot)]));
		links.push(self.trap);

		links
	}

	pub fn rebind_bindings(&mut self, producer: u32, live: &[u16]) {
		let dependency_count = to_port(self.dependencies.count());

		self.dependencies
			.set_all_from((0..dependency_count).map(|port| Link(producer, port)));

		for (offset, &slot) in (0..).zip(live) {
			self.locals[usize::from(slot)] = Link(producer, dependency_count + offset);
		}

		self.trap = Link(producer, dependency_count + to_port(live.len()));
	}

	fn handle_local_set(&mut self, instruction: LocalSet) {
		let LocalSet {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] = self.locals[usize::from(source)];
	}

	fn handle_i32_constant(&mut self, nodes: &mut Vec<Node>, instruction: I32Constant) {
		let I32Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = Node::add_i32_into(nodes, data);
	}

	fn handle_i64_constant(&mut self, nodes: &mut Vec<Node>, instruction: I64Constant) {
		let I64Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = Node::add_i64_into(nodes, data);
	}

	fn handle_f32_constant(&mut self, nodes: &mut Vec<Node>, instruction: F32Constant) {
		let F32Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = Node::add_f32_into(nodes, data);
	}

	fn handle_f64_constant(&mut self, nodes: &mut Vec<Node>, instruction: F64Constant) {
		let F64Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = Node::add_f64_into(nodes, data);
	}

	fn handle_ref_is_null(&mut self, nodes: &mut Vec<Node>, instruction: RefIsNull) {
		let RefIsNull {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			operation::RefIsNull::add_into(nodes, self.locals[usize::from(source)]);
	}

	fn handle_ref_null(&mut self, nodes: &mut Vec<Node>, instruction: RefNull) {
		let RefNull { destination } = instruction;

		self.locals[usize::from(destination)] = Node::add_null_into(nodes);
	}

	fn handle_ref_function(&mut self, nodes: &mut Vec<Node>, instruction: RefFunction) {
		let RefFunction {
			destination,
			function,
		} = instruction;

		let slot = self.dependencies.get(ReferenceType::Function, function);

		self.locals[usize::from(destination)] = operation::MutableGet::add_into(nodes, slot).0;
	}

	fn handle_unreachable(&mut self, nodes: &mut Vec<Node>) {
		self.trap = Node::add_trap_into(nodes);
	}

	fn handle_pre_call(
		&mut self,
		nodes: &mut Vec<Node>,
		closure: Link,
		from: u16,
		to: u16,
	) -> Vec<Link> {
		let mut arguments = Vec::with_capacity(usize::from(to - from) + 2);

		arguments.push(closure);
		arguments.extend_from_slice(&self.locals[usize::from(from)..usize::from(to)]);

		self.create_fence(nodes);

		arguments.push(self.trap);

		arguments
	}

	fn handle_post_call(&mut self, nodes: &mut Vec<Node>, call: u32, from: u16, to: u16) {
		let destinations = self.locals[usize::from(from)..usize::from(to)].iter_mut();

		for (port, destination) in (0..).zip(destinations) {
			*destination = Link(call, port);
		}

		self.trap = Link(call, to - from);

		self.create_fence(nodes);
	}

	fn handle_call(&mut self, nodes: &mut Vec<Node>, instruction: Call) {
		let Call {
			destinations,
			sources,
			function: callee,
		} = instruction;

		let closure = self.locals[usize::from(callee)];
		let function = operation::Extract::add_into(nodes, closure, 0);

		let arguments = self.handle_pre_call(nodes, closure, sources.0, sources.1);

		let call = operation::Apply::add_into(
			nodes,
			function,
			arguments,
			destinations.1 - destinations.0 + 1,
		);

		self.handle_post_call(nodes, call, destinations.0, destinations.1);
	}

	fn handle_global_get(&mut self, nodes: &mut Vec<Node>, instruction: GlobalGet) {
		let GlobalGet {
			destination,
			source,
		} = instruction;

		let state = self.dependencies.get(ReferenceType::Global, source);
		let (result, state) = operation::MutableGet::add_into(nodes, state);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Global, source, state);
	}

	fn handle_global_set(&mut self, nodes: &mut Vec<Node>, instruction: GlobalSet) {
		let GlobalSet {
			destination,
			source,
		} = instruction;

		let state = operation::MutableSet::add_into(
			nodes,
			self.dependencies.get(ReferenceType::Global, destination),
			self.locals[usize::from(source)],
		);

		self.dependencies
			.set(ReferenceType::Global, destination, state);
	}

	#[expect(
		clippy::too_many_lines,
		reason = "exhaustive match over instruction variants"
	)]
	fn handle_instruction(&mut self, nodes: &mut Vec<Node>, instruction: Instruction) {
		match instruction {
			Instruction::LocalSet(instruction) => self.handle_local_set(instruction),
			Instruction::LocalBranch(_) => {
				unreachable!("branch conditions are lowered by `run_branch`")
			}
			Instruction::I32Constant(instruction) => self.handle_i32_constant(nodes, instruction),
			Instruction::I64Constant(instruction) => self.handle_i64_constant(nodes, instruction),
			Instruction::F32Constant(instruction) => self.handle_f32_constant(nodes, instruction),
			Instruction::F64Constant(instruction) => self.handle_f64_constant(nodes, instruction),
			Instruction::RefIsNull(instruction) => self.handle_ref_is_null(nodes, instruction),
			Instruction::RefNull(instruction) => self.handle_ref_null(nodes, instruction),
			Instruction::RefFunction(instruction) => self.handle_ref_function(nodes, instruction),
			Instruction::Call(instruction) => self.handle_call(nodes, instruction),
			Instruction::Unreachable => self.handle_unreachable(nodes),
			Instruction::IntegerUnaryOperation(instruction) => {
				self.handle_integer_unary_operation(nodes, instruction);
			}
			Instruction::IntegerBinaryOperation(
				instruction @ IntegerBinaryOperation {
					operator:
						operation::integer::BinaryOperator::Divide { .. }
						| operation::integer::BinaryOperator::Remainder { .. },
					..
				},
			) => self.handle_trapping_integer_binary_operation(nodes, instruction),
			Instruction::IntegerBinaryOperation(instruction) => {
				self.handle_integer_binary_operation(nodes, instruction);
			}
			Instruction::IntegerCompareOperation(instruction) => {
				self.handle_integer_compare_operation(nodes, instruction);
			}
			Instruction::IntegerNarrow(instruction) => {
				self.handle_integer_narrow(nodes, instruction);
			}
			Instruction::IntegerWiden(instruction) => self.handle_integer_widen(nodes, instruction),
			Instruction::IntegerExtend(instruction) => {
				self.handle_integer_extend(nodes, instruction);
			}
			Instruction::IntegerConvertToNumber(instruction) => {
				self.handle_integer_convert_to_number(nodes, instruction);
			}
			Instruction::IntegerTransmuteToNumber(instruction) => {
				self.handle_integer_transmute_to_number(nodes, instruction);
			}
			Instruction::NumberUnaryOperation(instruction) => {
				self.handle_number_unary_operation(nodes, instruction);
			}
			Instruction::NumberBinaryOperation(instruction) => {
				self.handle_number_binary_operation(nodes, instruction);
			}
			Instruction::NumberCompareOperation(instruction) => {
				self.handle_number_compare_operation(nodes, instruction);
			}
			Instruction::NumberNarrow(instruction) => self.handle_number_narrow(nodes, instruction),
			Instruction::NumberWiden(instruction) => self.handle_number_widen(nodes, instruction),
			Instruction::NumberTruncateToInteger(
				instruction @ NumberTruncateToInteger {
					is_saturating: false,
					..
				},
			) => self.handle_trapping_number_truncate_to_integer(nodes, instruction),
			Instruction::NumberTruncateToInteger(instruction) => {
				self.handle_number_truncate_to_integer(nodes, instruction);
			}
			Instruction::NumberTransmuteToInteger(instruction) => {
				self.handle_number_transmute_to_integer(nodes, instruction);
			}
			Instruction::GlobalGet(instruction) => self.handle_global_get(nodes, instruction),
			Instruction::GlobalSet(instruction) => self.handle_global_set(nodes, instruction),
			Instruction::TableGet(instruction) => self.handle_table_get(nodes, instruction),
			Instruction::TableSet(instruction) => self.handle_table_set(nodes, instruction),
			Instruction::TableSize(instruction) => self.handle_table_size(nodes, instruction),
			Instruction::TableGrow(instruction) => self.handle_table_grow(nodes, instruction),
			Instruction::TableFill(instruction) => self.handle_table_fill(nodes, instruction),
			Instruction::TableCopy(instruction) => self.handle_table_copy(nodes, instruction),
			Instruction::TableInit(instruction) => self.handle_table_init(nodes, instruction),
			Instruction::ElementsDrop(instruction) => self.handle_elements_drop(nodes, instruction),
			Instruction::MemoryLoad(instruction) => self.handle_memory_load(nodes, instruction),
			Instruction::MemoryStore(instruction) => self.handle_memory_store(nodes, instruction),
			Instruction::MemorySize(instruction) => self.handle_memory_size(nodes, instruction),
			Instruction::MemoryGrow(instruction) => self.handle_memory_grow(nodes, instruction),
			Instruction::MemoryFill(instruction) => self.handle_memory_fill(nodes, instruction),
			Instruction::MemoryCopy(instruction) => self.handle_memory_copy(nodes, instruction),
			Instruction::MemoryInit(instruction) => self.handle_memory_init(nodes, instruction),
			Instruction::DataDrop(instruction) => self.handle_data_drop(nodes, instruction),
		}
	}

	pub fn run(&mut self, nodes: &mut Vec<Node>, instructions: &[Instruction]) {
		for &instruction in instructions {
			self.handle_instruction(nodes, instruction);
		}
	}

	pub fn run_branch(&mut self, nodes: &mut Vec<Node>, instructions: &[Instruction]) -> Link {
		let Some((&Instruction::LocalBranch(LocalBranch { source }), instructions)) =
			instructions.split_last()
		else {
			unreachable!("branch blocks end in a local branch")
		};

		self.run(nodes, instructions);

		self.locals[usize::from(source)]
	}
}
