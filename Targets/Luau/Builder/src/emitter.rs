use alloc::sync::Arc;
use core::any::Any;

use parking_lot::Mutex;

use ir_graph::{
	Link, Node,
	foreign::Foreign,
	operation::{self, Apply},
	region::{Branch, Function, Match, Repeat, repeat},
};
use luau_foreign::{
	Bit32And, Bit32ArShift, Bit32CountLz, Bit32CountRz, Bit32LRotate, Bit32LShift, Bit32Or,
	Bit32RRotate, Bit32RShift, Bit32Xor, BooleanToInteger, BufferLength, BufferLoad, BufferStore,
	FlipMostSignificant, FromBitsF32, FromBitsI64, IntoBitsF32, IntoBitsI64, IsPositive, LuauAdd,
	LuauDivide, LuauEqual, LuauFloorDivide, LuauLessThan, LuauLessThanEqual, LuauModulo,
	LuauMultiply, LuauNegate, LuauNotEqual, LuauSubtract, MathAbs, MathCeil, MathFloor, MathFmod,
	MathMax, MathMin, MathModf, MathSqrt, TableLength, TableLoad, TableStore, VectorCreate,
	VectorX,
};
use luau_tree::{
	expression::{self, Apply as ApplyExpression, Call as CallExpression, Expression, Local, Name},
	statement::Sequence,
};
use turing_machine_foreign::Tell as TuringTell;
use web_assembly_foreign::Export as WasmExport;

use super::{
	code_handler::CodeHandler,
	data_handler::{DataHandler, build_foreign, build_match_expression, memory_store_name},
	policy::{LuauPolicy, PHYSICAL_REGISTERS},
};

pub struct Emitter<'allocator, 'policy> {
	allocator: &'allocator mut ir_allocator::Allocator,
	policy: &'policy LuauPolicy,
	code_handler: CodeHandler,
	data_handler: DataHandler,

	region: u32,
	next_region: u32,
}

fn fast_locals_for(peak: u32, argument_count: u16) -> Vec<Name> {
	let fast_count = peak.min(PHYSICAL_REGISTERS);
	let start = u32::from(argument_count);

	(start..fast_count).map(|id| Name { id }).collect()
}

fn stack_size_class_for(peak: u32) -> u16 {
	const SIZE_CLASSES: [u16; 7] = [0, 16, 64, 256, 1024, 4096, 16384];

	let spill = peak.saturating_sub(PHYSICAL_REGISTERS);

	SIZE_CLASSES
		.into_iter()
		.find(|&class| u32::from(class) >= spill)
		.unwrap_or_else(|| panic!("spill {spill} exceeds maximum size class"))
}

fn collect_argument_names(argument_count: u16) -> Vec<Name> {
	(0..u32::from(argument_count))
		.map(|id| Name { id })
		.collect()
}

impl<'allocator, 'policy> Emitter<'allocator, 'policy> {
	#[must_use]
	pub fn new(
		allocator: &'allocator mut ir_allocator::Allocator,
		policy: &'policy LuauPolicy,
	) -> Self {
		Self {
			allocator,
			policy,
			code_handler: CodeHandler::new(),
			data_handler: DataHandler::new(),
			region: 0,
			next_region: 0,
		}
	}

	fn emit_assignment(&mut self, destination: u32, source: Expression) {
		let local = self
			.data_handler
			.local_of(self.region, Link(destination, 0));

		self.code_handler.emit_assign(local, source);
	}

	fn emit_expression(&mut self, nodes: &[Node], id: u32) {
		let expression = self.expression_of(nodes, id);

		self.emit_or_defer(id, expression);
	}

	fn emit_or_defer(&mut self, id: u32, expression: Expression) {
		if self.data_handler.is_deferred(self.region, id) {
			self.data_handler.store(self.region, id, expression);
		} else {
			self.emit_assignment(id, expression);
		}
	}

	fn assert_region_nodes(&self, region: u32, nodes: &[Node]) {
		debug_assert_eq!(
			self.data_handler.node_count(region),
			nodes.len(),
			"emitter and allocator must traverse regions in one order"
		);
	}

	pub fn emit_function(&mut self, function: &Function) -> expression::Function {
		let (arena, peak) = self.allocator.run(self.policy, &function.nodes);

		self.data_handler.install(arena);

		self.assert_region_nodes(0, &function.nodes);

		self.region = 0;
		self.next_region = 1;
		self.code_handler.push_scope();

		self.handle_nodes(&function.nodes);

		let arguments = collect_argument_names(function.argument_count);
		let locals = fast_locals_for(peak, function.argument_count);
		let stack = stack_size_class_for(peak);
		let code = self.code_handler.pop_scope();
		let returns = self
			.data_handler
			.load_all(self.region, &function.results().sources);

		expression::Function {
			arguments,
			locals,
			stack,
			code,
			returns,
		}
	}

	fn build_function(&mut self, arc: &Arc<Mutex<Function>>) -> Expression {
		let function = arc.lock();

		let mut child = Emitter::new(&mut *self.allocator, self.policy);
		let inner = child.emit_function(&function);

		drop(function);

		Expression::Function(inner.into())
	}

	fn emit_transfer(&mut self, destinations: &[Local], sources: &[Link]) {
		let pairs = destinations
			.iter()
			.zip(sources)
			.map(|(&destination, &source)| {
				(destination, self.data_handler.local_of(self.region, source))
			})
			.collect();

		self.code_handler.emit_local_moves(pairs);
	}

	// A node's forwarded state port carries its operand's value as the post-operation state.
	// The operation then reads from the materialized state port, never the raw operand, so the
	// builder owns the port whatever register the allocator gives it.
	fn bridge(&mut self, id: u32, state_port: u16, source: Link) -> Link {
		let state_link = Link(id, state_port);
		let destination = self.data_handler.local_of(self.region, state_link);

		self.emit_transfer(&[destination], &[source]);

		state_link
	}

	fn handle_branch(
		&mut self,
		branch_arc: &Arc<Mutex<Branch>>,
		result_locals: &[Local],
	) -> Sequence {
		let branch = branch_arc.lock();
		let branch_region = self.next_region;

		self.next_region += 1;
		self.assert_region_nodes(branch_region, &branch.nodes);

		self.region = branch_region;
		self.code_handler.push_scope();

		self.handle_nodes(&branch.nodes);

		self.emit_transfer(result_locals, &branch.results().sources);

		drop(branch);

		self.code_handler.pop_scope()
	}

	fn resolve_condition(&mut self, condition: Link, branch_count: usize) -> Expression {
		let condition = self.data_handler.load(self.region, condition);

		if branch_count == 2 {
			condition.into_boolean()
		} else {
			condition
		}
	}

	fn handle_match(&mut self, id: u32, arc: &Arc<Mutex<Match>>) {
		let matcher = arc.lock();
		let parent_region = self.region;

		let result_count = matcher.result_count();
		let result_locals = self
			.data_handler
			.port_locals(parent_region, id, result_count);

		let branches: Vec<_> = matcher
			.branches
			.iter()
			.map(|branch_arc| self.handle_branch(branch_arc, &result_locals))
			.collect();

		self.region = parent_region;

		let condition = self.resolve_condition(matcher.condition, branches.len());

		drop(matcher);

		if let Some(destination) = Sequence::as_branch_destination(&branches) {
			let source = build_match_expression(condition, branches);

			self.code_handler.emit_assign(destination, source);
			return;
		}

		self.code_handler.emit_match(condition, branches);
	}

	fn handle_repeat(&mut self, arc: &Arc<Mutex<Repeat>>) {
		let repeat = arc.lock();
		let repeat_region = self.next_region;
		let parent_region = self.region;

		self.next_region += 1;
		self.assert_region_nodes(repeat_region, &repeat.nodes);

		let count = u16::try_from(repeat.arguments.len()).unwrap();
		let slot_locals = self
			.data_handler
			.port_locals(repeat_region, Repeat::ARGUMENTS_ID, count);

		self.emit_transfer(&slot_locals, &repeat.arguments);

		self.region = repeat_region;
		self.code_handler.push_scope();

		self.handle_nodes(&repeat.nodes);

		drop(repeat);

		self.region = parent_region;
	}

	fn handle_repeat_results(&mut self, node: &repeat::Results) {
		let count = u16::try_from(node.sources.len()).unwrap();
		let slot_locals = self
			.data_handler
			.port_locals(self.region, Repeat::ARGUMENTS_ID, count);
		let condition = self.data_handler.load(self.region, node.condition);

		self.code_handler.push_scope();
		self.emit_transfer(&slot_locals, &node.sources);
		let rotation = self.code_handler.pop_scope();

		self.code_handler.emit_repeat(condition, rotation);
	}

	// Identity and fence are pure forwarders: every output port carries an operand's value,
	// so each port is assigned from its source as an explicit move.
	fn handle_identity(&mut self, id: u32, node: &operation::Identity) {
		let destinations = self
			.data_handler
			.port_locals(self.region, id, node.result_count());

		self.emit_transfer(&destinations, &node.sources);
	}

	fn handle_fence(&mut self, id: u32, node: &operation::Fence) {
		let destinations = self
			.data_handler
			.port_locals(self.region, id, node.result_count());

		self.emit_transfer(&destinations, &node.sources);
	}

	fn handle_wasm_export(&mut self, node: &WasmExport) {
		let value = self.data_handler.load(self.region, node.value);
		let identifier = Expression::String(Arc::clone(&node.identifier));
		let expression = ApplyExpression {
			name: "rt_export",
			arguments: [identifier, value],
		};

		self.code_handler
			.emit_call(Vec::new(), Expression::Apply2Arguments(expression.into()));
	}

	fn handle_turing_tell(&mut self, node: TuringTell) {
		let value =
			self.data_handler
				.build_apply_1(self.region, "rt_turing_tell", [node.character]);

		self.code_handler.emit_call(Vec::new(), value);
	}

	fn handle_foreign(&mut self, nodes: &[Node], id: u32, foreign: &dyn Foreign) {
		let any: &dyn Any = foreign;

		if let Some(node) = any.downcast_ref::<WasmExport>() {
			self.handle_wasm_export(node);
		} else if let Some(&node) = any.downcast_ref::<TuringTell>() {
			self.handle_turing_tell(node);
		} else if let Some(&node) = any.downcast_ref::<BufferStore>() {
			self.handle_buffer_store(id, node);
		} else if let Some(&node) = any.downcast_ref::<TableStore>() {
			self.handle_table_store(id, node);
		} else if let Some(&node) = any.downcast_ref::<FromBitsI64>()
			&& !self.data_handler.is_deferred(self.region, id)
		{
			self.handle_from_bits_i64(id, node);
		} else {
			self.emit_expression(nodes, id);
		}
	}

	fn handle_buffer_store(&mut self, id: u32, node: BufferStore) {
		let reference = self.bridge(id, BufferStore::STATE_PORT, node.reference);
		let value = self
			.data_handler
			.build_buffer_store(self.region, reference, node);

		self.code_handler.emit_call(Vec::new(), value);
	}

	fn handle_table_store(&mut self, id: u32, node: TableStore) {
		let reference = self.bridge(id, TableStore::STATE_PORT, node.reference);
		let table = self.data_handler.load(self.region, reference);
		let offset = self.data_handler.load(self.region, node.offset);
		let value = self.data_handler.load(self.region, node.value);

		self.code_handler.emit_set_index(table, offset, value);
	}

	fn handle_from_bits_i64(&mut self, id: u32, node: FromBitsI64) {
		let results = self.data_handler.port_locals(self.region, id, 2);
		let value = self
			.data_handler
			.build_apply_1(self.region, "from_bits_i64", [node.source]);

		self.code_handler.emit_call(results, value);
	}

	fn value_foreign(&mut self, foreign: &dyn Foreign) -> Expression {
		let any: &dyn Any = foreign;

		if let Some(expression) = self.value_bit32(any) {
			return expression;
		}

		if let Some(expression) = self.value_operator(any) {
			return expression;
		}

		if let Some(expression) = self.value_transmute(any) {
			return expression;
		}

		if let Some(expression) = self.value_math(any) {
			return expression;
		}

		if let Some(expression) = self.value_buffer(any) {
			return expression;
		}

		if let Some(expression) = self.value_vector(any) {
			return expression;
		}

		if let Some(expression) = self.value_table(any) {
			return expression;
		}

		build_foreign(foreign)
	}

	fn value_vector(&mut self, any: &dyn Any) -> Option<Expression> {
		let region = self.region;

		if let Some(node) = any.downcast_ref::<VectorCreate>() {
			return Some(self.data_handler.build_vector_create(region, node.source));
		}

		if let Some(node) = any.downcast_ref::<VectorX>() {
			return Some(self.data_handler.build_vector_x(region, node.source));
		}

		None
	}

	fn value_table(&mut self, any: &dyn Any) -> Option<Expression> {
		let region = self.region;

		if let Some(node) = any.downcast_ref::<TableLoad>() {
			return Some(
				self.data_handler
					.build_index(region, node.reference, node.offset),
			);
		}

		if let Some(node) = any.downcast_ref::<TableLength>() {
			return Some(self.data_handler.build_table_length(region, node.source));
		}

		None
	}

	fn value_buffer(&mut self, any: &dyn Any) -> Option<Expression> {
		let region = self.region;

		if let Some(node) = any.downcast_ref::<BufferLoad>() {
			return Some(self.data_handler.build_apply_2(
				region,
				node.name,
				[node.buffer, node.offset],
			));
		}

		if let Some(node) = any.downcast_ref::<BufferLength>() {
			return Some(self.data_handler.build_buffer_length(region, node.source));
		}

		None
	}

	fn value_bit32(&mut self, any: &dyn Any) -> Option<Expression> {
		let region = self.region;

		let expression = if let Some(node) = any.downcast_ref::<Bit32And>() {
			self.data_handler
				.build_apply_2(region, "bit32_and", [node.lhs, node.rhs])
		} else if let Some(node) = any.downcast_ref::<Bit32Or>() {
			self.data_handler
				.build_apply_2(region, "bit32_or", [node.lhs, node.rhs])
		} else if let Some(node) = any.downcast_ref::<Bit32Xor>() {
			self.data_handler
				.build_apply_2(region, "bit32_xor", [node.lhs, node.rhs])
		} else if let Some(node) = any.downcast_ref::<Bit32LShift>() {
			self.data_handler
				.build_apply_2(region, "bit32_lshift", [node.lhs, node.rhs])
		} else if let Some(node) = any.downcast_ref::<Bit32RShift>() {
			self.data_handler
				.build_apply_2(region, "bit32_rshift", [node.lhs, node.rhs])
		} else if let Some(node) = any.downcast_ref::<Bit32ArShift>() {
			self.data_handler
				.build_apply_2(region, "bit32_arshift", [node.lhs, node.rhs])
		} else if let Some(node) = any.downcast_ref::<Bit32LRotate>() {
			self.data_handler
				.build_apply_2(region, "bit32_lrotate", [node.lhs, node.rhs])
		} else if let Some(node) = any.downcast_ref::<Bit32RRotate>() {
			self.data_handler
				.build_apply_2(region, "bit32_rrotate", [node.lhs, node.rhs])
		} else if let Some(node) = any.downcast_ref::<Bit32CountLz>() {
			self.data_handler
				.build_apply_1(region, "bit32_countlz", [node.source])
		} else if let Some(node) = any.downcast_ref::<Bit32CountRz>() {
			self.data_handler
				.build_apply_1(region, "bit32_countrz", [node.source])
		} else {
			return None;
		};

		Some(expression)
	}

	fn value_operator(&mut self, any: &dyn Any) -> Option<Expression> {
		let region = self.region;

		let expression = if let Some(node) = any.downcast_ref::<LuauAdd>() {
			self.data_handler
				.build_infix(region, "+", node.lhs, node.rhs)
		} else if let Some(node) = any.downcast_ref::<LuauSubtract>() {
			self.data_handler
				.build_infix(region, "-", node.lhs, node.rhs)
		} else if let Some(node) = any.downcast_ref::<LuauMultiply>() {
			self.data_handler
				.build_infix(region, "*", node.lhs, node.rhs)
		} else if let Some(node) = any.downcast_ref::<LuauDivide>() {
			self.data_handler
				.build_infix(region, "/", node.lhs, node.rhs)
		} else if let Some(node) = any.downcast_ref::<LuauFloorDivide>() {
			self.data_handler
				.build_infix(region, "//", node.lhs, node.rhs)
		} else if let Some(node) = any.downcast_ref::<LuauModulo>() {
			self.data_handler
				.build_infix(region, "%", node.lhs, node.rhs)
		} else if let Some(node) = any.downcast_ref::<LuauNegate>() {
			self.data_handler.build_prefix(region, "-", node.source)
		} else if let Some(node) = any.downcast_ref::<LuauEqual>() {
			self.data_handler
				.build_infix(region, "==", node.lhs, node.rhs)
		} else if let Some(node) = any.downcast_ref::<LuauNotEqual>() {
			self.data_handler
				.build_infix(region, "~=", node.lhs, node.rhs)
		} else if let Some(node) = any.downcast_ref::<LuauLessThan>() {
			self.data_handler
				.build_infix(region, "<", node.lhs, node.rhs)
		} else if let Some(node) = any.downcast_ref::<LuauLessThanEqual>() {
			self.data_handler
				.build_infix(region, "<=", node.lhs, node.rhs)
		} else {
			return None;
		};

		Some(expression)
	}

	fn value_transmute(&mut self, any: &dyn Any) -> Option<Expression> {
		let region = self.region;

		let expression = if let Some(node) = any.downcast_ref::<IntoBitsI64>() {
			self.data_handler
				.build_apply_2(region, "into_bits_i64", [node.lhs, node.rhs])
		} else if let Some(node) = any.downcast_ref::<FromBitsI64>() {
			self.data_handler
				.build_apply_1(region, "from_bits_i64", [node.source])
		} else if let Some(node) = any.downcast_ref::<FlipMostSignificant>() {
			self.data_handler
				.build_apply_1(region, "flip_most_significant", [node.source])
		} else if let Some(node) = any.downcast_ref::<FromBitsF32>() {
			self.data_handler
				.build_apply_1(region, "from_bits_f32", [node.source])
		} else if let Some(node) = any.downcast_ref::<IntoBitsF32>() {
			self.data_handler
				.build_apply_1(region, "into_bits_f32", [node.source])
		} else if let Some(node) = any.downcast_ref::<IsPositive>() {
			self.data_handler
				.build_apply_1(region, "is_positive", [node.source])
		} else if let Some(node) = any.downcast_ref::<BooleanToInteger>() {
			self.data_handler
				.build_boolean_to_integer(region, node.source)
		} else {
			return None;
		};

		Some(expression)
	}

	fn value_math(&mut self, any: &dyn Any) -> Option<Expression> {
		let region = self.region;

		let expression = if let Some(node) = any.downcast_ref::<MathAbs>() {
			self.data_handler
				.build_apply_1(region, "math_abs", [node.source])
		} else if let Some(node) = any.downcast_ref::<MathSqrt>() {
			self.data_handler
				.build_apply_1(region, "math_sqrt", [node.source])
		} else if let Some(node) = any.downcast_ref::<MathCeil>() {
			self.data_handler
				.build_apply_1(region, "math_ceil", [node.source])
		} else if let Some(node) = any.downcast_ref::<MathFloor>() {
			self.data_handler
				.build_apply_1(region, "math_floor", [node.source])
		} else if let Some(node) = any.downcast_ref::<MathModf>() {
			self.data_handler
				.build_apply_1(region, "math_modf", [node.source])
		} else if let Some(node) = any.downcast_ref::<MathMin>() {
			self.data_handler
				.build_apply_2(region, "math_min", [node.lhs, node.rhs])
		} else if let Some(node) = any.downcast_ref::<MathMax>() {
			self.data_handler
				.build_apply_2(region, "math_max", [node.lhs, node.rhs])
		} else if let Some(node) = any.downcast_ref::<MathFmod>() {
			self.data_handler
				.build_apply_2(region, "math_fmod", [node.lhs, node.rhs])
		} else {
			return None;
		};

		Some(expression)
	}

	fn handle_call(&mut self, id: u32, node: &Apply) {
		let function = self.data_handler.load(self.region, node.function);
		let arguments = self.data_handler.load_all(self.region, &node.arguments);
		let results = self
			.data_handler
			.port_locals(self.region, id, node.result_count);
		let call = Expression::Call(
			CallExpression {
				function,
				arguments,
			}
			.into(),
		);

		self.code_handler.emit_call(results, call);
	}

	#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
	fn expression_of(&mut self, nodes: &[Node], id: u32) -> Expression {
		match &nodes[usize::try_from(id).unwrap()] {
			Node::Function(arc) => self.build_function(arc),

			Node::Match(_)
			| Node::Repeat(_)
			| Node::FunctionArguments(_)
			| Node::FunctionResults(_)
			| Node::BranchArguments(_)
			| Node::BranchResults(_)
			| Node::RepeatArguments(_)
			| Node::RepeatResults(_)
			| Node::Identity(_)
			| Node::Fence(_)
			| Node::Apply(_)
			| Node::MutableGet(_)
			| Node::MutableSet(_)
			| Node::TableGet(_)
			| Node::TableSet(_)
			| Node::TableSize(_)
			| Node::TableGrow(_)
			| Node::TableFill(_)
			| Node::TableCopy(_)
			| Node::TableDrop(_)
			| Node::MemoryLoad(_)
			| Node::MemorySize(_)
			| Node::MemoryGrow(_)
			| Node::MemoryStore(_)
			| Node::MemoryFill(_)
			| Node::MemoryCopy(_)
			| Node::MemoryDrop(_) => unreachable!("statements are never rebuilt as expressions"),

			Node::Foreign(foreign) => self.value_foreign(foreign.as_ref()),

			Node::Trap => Expression::Trap,
			Node::Null => Expression::Null,
			Node::I32(value) => Expression::I32(*value),
			Node::I64(value) => Expression::I64(*value),
			Node::F32(value) => Expression::F32(*value),
			Node::F64(value) => Expression::F64(*value),

			Node::RefIsNull(node) => self.data_handler.build_ref_is_null(self.region, *node),

			Node::IntegerUnaryOperation(node) => self
				.data_handler
				.build_integer_unary_operation(self.region, *node),
			Node::IntegerBinaryOperation(node) => self
				.data_handler
				.build_integer_binary_operation(self.region, *node),
			Node::IntegerCompareOperation(node) => self
				.data_handler
				.build_integer_compare_operation(self.region, *node),
			Node::IntegerNarrow(node) => self.data_handler.build_integer_narrow(self.region, *node),
			Node::IntegerWiden(node) => self.data_handler.build_integer_widen(self.region, *node),
			Node::IntegerSignExtend(node) => self
				.data_handler
				.build_integer_sign_extend(self.region, *node),
			Node::IntegerConvertToNumber(node) => self
				.data_handler
				.build_integer_convert_to_number(self.region, *node),
			Node::IntegerTransmuteToNumber(node) => self
				.data_handler
				.build_integer_transmute_to_number(self.region, *node),

			Node::NumberUnaryOperation(node) => self
				.data_handler
				.build_number_unary_operation(self.region, *node),
			Node::NumberBinaryOperation(node) => self
				.data_handler
				.build_number_binary_operation(self.region, *node),
			Node::NumberCompareOperation(node) => self
				.data_handler
				.build_number_compare_operation(self.region, *node),
			Node::NumberNarrow(node) => self.data_handler.build_number_narrow(self.region, *node),
			Node::NumberWiden(node) => self.data_handler.build_number_widen(self.region, *node),
			Node::NumberTruncateToInteger(node) => self
				.data_handler
				.build_number_truncate_to_integer(self.region, *node),
			Node::NumberTransmuteToInteger(node) => self
				.data_handler
				.build_number_transmute_to_integer(self.region, *node),

			Node::MutableNew(node) => self.data_handler.build_mutable_new(self.region, *node),

			Node::Aggregate(node) => self.data_handler.build_aggregate(self.region, node),
			Node::Extract(node) => self.data_handler.build_extract(self.region, *node),

			Node::TableNew(node) => self.data_handler.build_table_new(self.region, node),

			Node::MemoryNew(node) => Expression::MemoryNew(node.clone()),
		}
	}

	fn handle_mutable_get(&mut self, id: u32, node: operation::MutableGet) {
		let reference = self.bridge(id, operation::MutableGet::STATE_PORT, node.source);
		let value = self.data_handler.build_mutable_get(self.region, reference);

		self.emit_or_defer(id, value);
	}

	// A mutable cell is a one-field aggregate, so a write to it sets index one.
	fn handle_mutable_set(&mut self, id: u32, node: operation::MutableSet) {
		let reference = self.bridge(id, operation::MutableSet::STATE_PORT, node.destination);
		let destination = self.data_handler.load(self.region, reference);
		let source = self.data_handler.load(self.region, node.source);

		self.code_handler
			.emit_set_index(destination, Expression::I32(1), source);
	}

	fn handle_table_get(&mut self, id: u32, node: operation::TableGet) {
		let reference = self.bridge(id, operation::TableGet::STATE_PORT, node.source.reference);
		let value = self
			.data_handler
			.build_table_get(self.region, reference, node.source.offset);

		self.emit_or_defer(id, value);
	}

	fn handle_table_set(&mut self, id: u32, node: operation::TableSet) {
		let reference = self.bridge(
			id,
			operation::TableSet::STATE_PORT,
			node.destination.reference,
		);
		let value = self.data_handler.build_apply_3(
			self.region,
			"rt_table_set",
			[reference, node.destination.offset, node.source],
		);

		self.code_handler.emit_call(Vec::new(), value);
	}

	fn handle_table_size(&mut self, id: u32, node: operation::TableSize) {
		let reference = self.bridge(id, operation::TableSize::STATE_PORT, node.source);
		let value = self.data_handler.build_table_size(self.region, reference);

		self.emit_or_defer(id, value);
	}

	fn handle_table_grow(&mut self, id: u32, node: operation::TableGrow) {
		let reference = self.bridge(id, operation::TableGrow::STATE_PORT, node.destination);
		let value =
			self.data_handler
				.build_table_grow(self.region, reference, node.initializer, node.size);

		self.emit_or_defer(id, value);
	}

	fn handle_table_fill(&mut self, id: u32, node: operation::TableFill) {
		let reference = self.bridge(
			id,
			operation::TableFill::STATE_PORT,
			node.destination.reference,
		);
		let value = self.data_handler.build_apply_4(
			self.region,
			"rt_table_fill",
			[reference, node.destination.offset, node.source, node.size],
		);

		self.code_handler.emit_call(Vec::new(), value);
	}

	fn handle_table_copy(&mut self, id: u32, node: operation::TableCopy) {
		let destination = Link(id, operation::TableCopy::DESTINATION_STATE_PORT);
		let source = Link(id, operation::TableCopy::SOURCE_STATE_PORT);
		let destination_local = self.data_handler.local_of(self.region, destination);
		let source_local = self.data_handler.local_of(self.region, source);

		self.emit_transfer(
			&[destination_local, source_local],
			&[node.destination.reference, node.source.reference],
		);

		let value = self.data_handler.build_apply_5(
			self.region,
			"rt_table_copy",
			[
				destination,
				node.destination.offset,
				source,
				node.source.offset,
				node.size,
			],
		);

		self.code_handler.emit_call(Vec::new(), value);
	}

	fn handle_table_drop(&mut self, id: u32, node: operation::TableDrop) {
		let reference = self.bridge(id, operation::TableDrop::STATE_PORT, node.source);
		let value = self
			.data_handler
			.build_apply_1(self.region, "rt_table_drop", [reference]);

		self.code_handler.emit_call(Vec::new(), value);
	}

	fn handle_memory_load(&mut self, id: u32, node: operation::MemoryLoad) {
		let reference = self.bridge(id, operation::MemoryLoad::STATE_PORT, node.source.reference);
		let value = self.data_handler.build_memory_load(
			self.region,
			reference,
			node.source.offset,
			node.kind,
		);

		self.emit_or_defer(id, value);
	}

	fn handle_memory_store(&mut self, id: u32, node: operation::MemoryStore) {
		let reference = self.bridge(
			id,
			operation::MemoryStore::STATE_PORT,
			node.destination.reference,
		);
		let value = self.data_handler.build_apply_3(
			self.region,
			memory_store_name(node.kind),
			[reference, node.destination.offset, node.source],
		);

		self.code_handler.emit_call(Vec::new(), value);
	}

	fn handle_memory_size(&mut self, id: u32, node: operation::MemorySize) {
		let reference = self.bridge(id, operation::MemorySize::STATE_PORT, node.source);
		let value = self.data_handler.build_memory_size(self.region, reference);

		self.emit_or_defer(id, value);
	}

	fn handle_memory_grow(&mut self, id: u32, node: operation::MemoryGrow) {
		let reference = self.bridge(id, operation::MemoryGrow::STATE_PORT, node.destination);
		let value = self
			.data_handler
			.build_memory_grow(self.region, reference, node.size);

		self.emit_or_defer(id, value);
	}

	fn handle_memory_fill(&mut self, id: u32, node: operation::MemoryFill) {
		let reference = self.bridge(
			id,
			operation::MemoryFill::STATE_PORT,
			node.destination.reference,
		);
		let value = self.data_handler.build_apply_4(
			self.region,
			"rt_memory_fill",
			[reference, node.destination.offset, node.byte, node.size],
		);

		self.code_handler.emit_call(Vec::new(), value);
	}

	fn handle_memory_copy(&mut self, id: u32, node: operation::MemoryCopy) {
		let destination = Link(id, operation::MemoryCopy::DESTINATION_STATE_PORT);
		let source = Link(id, operation::MemoryCopy::SOURCE_STATE_PORT);
		let destination_local = self.data_handler.local_of(self.region, destination);
		let source_local = self.data_handler.local_of(self.region, source);

		self.emit_transfer(
			&[destination_local, source_local],
			&[node.destination.reference, node.source.reference],
		);

		let value = self.data_handler.build_apply_5(
			self.region,
			"rt_memory_copy",
			[
				destination,
				node.destination.offset,
				source,
				node.source.offset,
				node.size,
			],
		);

		self.code_handler.emit_call(Vec::new(), value);
	}

	fn handle_memory_drop(&mut self, id: u32, node: operation::MemoryDrop) {
		let reference = self.bridge(id, operation::MemoryDrop::STATE_PORT, node.source);
		let value = self
			.data_handler
			.build_apply_1(self.region, "rt_memory_drop", [reference]);

		self.code_handler.emit_call(Vec::new(), value);
	}

	#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
	fn handle_node(&mut self, nodes: &[Node], id: u32, node: &Node) {
		match *node {
			Node::Function(ref arc) => {
				let expression = self.build_function(arc);

				self.emit_or_defer(id, expression);
			}
			Node::Match(ref arc) => self.handle_match(id, arc),
			Node::Repeat(ref arc) => self.handle_repeat(arc),

			Node::FunctionArguments(_)
			| Node::FunctionResults(_)
			| Node::BranchArguments(_)
			| Node::BranchResults(_)
			| Node::RepeatArguments(_) => {}

			Node::RepeatResults(ref node) => self.handle_repeat_results(node),

			Node::Identity(ref node) => self.handle_identity(id, node),
			Node::Fence(ref node) => self.handle_fence(id, node),

			Node::Foreign(ref node) => self.handle_foreign(nodes, id, node.as_ref()),

			Node::Trap
			| Node::Null
			| Node::I32(_)
			| Node::I64(_)
			| Node::F32(_)
			| Node::F64(_)
			| Node::RefIsNull(_)
			| Node::IntegerUnaryOperation(_)
			| Node::IntegerBinaryOperation(_)
			| Node::IntegerCompareOperation(_)
			| Node::IntegerNarrow(_)
			| Node::IntegerWiden(_)
			| Node::IntegerSignExtend(_)
			| Node::IntegerConvertToNumber(_)
			| Node::IntegerTransmuteToNumber(_)
			| Node::NumberUnaryOperation(_)
			| Node::NumberBinaryOperation(_)
			| Node::NumberCompareOperation(_)
			| Node::NumberNarrow(_)
			| Node::NumberWiden(_)
			| Node::NumberTruncateToInteger(_)
			| Node::NumberTransmuteToInteger(_)
			| Node::MutableNew(_)
			| Node::Aggregate(_)
			| Node::Extract(_)
			| Node::TableNew(_)
			| Node::MemoryNew(_) => self.emit_expression(nodes, id),

			Node::Apply(ref node) => self.handle_call(id, node),
			Node::MutableGet(node) => self.handle_mutable_get(id, node),
			Node::MutableSet(node) => self.handle_mutable_set(id, node),
			Node::TableGet(node) => self.handle_table_get(id, node),
			Node::TableSet(node) => self.handle_table_set(id, node),
			Node::TableSize(node) => self.handle_table_size(id, node),
			Node::TableGrow(node) => self.handle_table_grow(id, node),
			Node::TableFill(node) => self.handle_table_fill(id, node),
			Node::TableCopy(node) => self.handle_table_copy(id, node),
			Node::TableDrop(node) => self.handle_table_drop(id, node),
			Node::MemoryLoad(node) => self.handle_memory_load(id, node),
			Node::MemoryStore(node) => self.handle_memory_store(id, node),
			Node::MemorySize(node) => self.handle_memory_size(id, node),
			Node::MemoryGrow(node) => self.handle_memory_grow(id, node),
			Node::MemoryFill(node) => self.handle_memory_fill(id, node),
			Node::MemoryCopy(node) => self.handle_memory_copy(id, node),
			Node::MemoryDrop(node) => self.handle_memory_drop(id, node),
		}
	}

	fn handle_nodes(&mut self, nodes: &[Node]) {
		for (id, node) in nodes.iter().enumerate() {
			let id = id.try_into().unwrap();

			self.handle_node(nodes, id, node);
		}
	}
}
