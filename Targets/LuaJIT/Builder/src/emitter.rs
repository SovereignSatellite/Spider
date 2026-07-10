use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::{
	Link, Node,
	operation::{self, Apply, Export},
	region::{Branch, Function, Match, Repeat, repeat},
};
use luajit_tree::{
	expression::{self, Expression, Local, Name},
	statement::Sequence,
};

use super::{
	code_handler::CodeHandler,
	data_handler::{DataHandler, build_import},
	policy::{LuaJITPolicy, PHYSICAL_REGISTERS},
};

pub struct Emitter<'allocator, 'policy> {
	allocator: &'allocator mut ir_allocator::Allocator,
	policy: &'policy LuaJITPolicy,
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
		policy: &'policy LuaJITPolicy,
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
		assert!(
			u32::from(function.argument_count) <= PHYSICAL_REGISTERS,
			"argument count {} exceeds the physical register count {PHYSICAL_REGISTERS}",
			function.argument_count
		);

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

	fn handle_export(&mut self, id: u32, node: &Export) {
		let state = self.bridge(id, Export::STATE_PORT, node.value);
		let value = self.data_handler.load(self.region, state);
		let identifier = Expression::String(Arc::clone(&node.identifier));

		self.code_handler
			.emit_runtime_call("export", vec![identifier, value]);
	}

	fn handle_call(&mut self, id: u32, node: &Apply) {
		let function = self.data_handler.load(self.region, node.function);
		let arguments = self.data_handler.load_all(self.region, &node.arguments);
		let results = self
			.data_handler
			.port_locals(self.region, id, node.result_count);

		self.code_handler.emit_call(function, results, arguments);
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
			| Node::Export(_)
			| Node::Identity(_)
			| Node::Fence(_)
			| Node::Apply(_)
			| Node::MutableGet(_)
			| Node::MutableSet(_)
			| Node::TableSet(_)
			| Node::TableFill(_)
			| Node::TableCopy(_)
			| Node::TableDrop(_)
			| Node::MemoryLoad(_)
			| Node::MemoryStore(_)
			| Node::MemoryFill(_)
			| Node::MemoryCopy(_)
			| Node::MemoryDrop(_) => unreachable!("statements are never rebuilt as expressions"),

			Node::Import(node) => build_import(node),

			Node::Foreign(_) => unreachable!("the LuaJIT target consumes no foreign nodes"),

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
			Node::TableGet(node) => self.data_handler.build_table_get(self.region, *node),
			Node::TableSize(node) => self.data_handler.build_table_size(self.region, *node),
			Node::TableGrow(node) => self.data_handler.build_table_grow(self.region, *node),

			Node::MemoryNew(node) => self.data_handler.build_memory_new(self.region, node),
		}
	}

	fn handle_mutable_get(&mut self, id: u32, node: operation::MutableGet) {
		let reference = self.bridge(id, operation::MutableGet::STATE_PORT, node.source);
		let value = self.data_handler.build_mutable_get(self.region, reference);

		self.emit_or_defer(id, value);
	}

	fn handle_mutable_set(&mut self, id: u32, node: operation::MutableSet) {
		let destination = self.bridge(id, operation::MutableSet::STATE_PORT, node.destination);
		let destination = self.data_handler.load(self.region, destination);
		let source = self.data_handler.load(self.region, node.source);

		self.code_handler.emit_mutable_set(destination, source);
	}

	fn handle_table_set(&mut self, node: operation::TableSet) {
		let destination = self
			.data_handler
			.load_location(self.region, node.destination);
		let source = self.data_handler.load(self.region, node.source);

		self.code_handler.emit_table_set(destination, source);
	}

	fn handle_table_fill(&mut self, node: operation::TableFill) {
		let destination = self
			.data_handler
			.load_location(self.region, node.destination);
		let source = self.data_handler.load(self.region, node.source);
		let size = self.data_handler.load(self.region, node.size);

		self.code_handler.emit_table_fill(destination, source, size);
	}

	fn handle_table_copy(&mut self, node: operation::TableCopy) {
		let destination = self
			.data_handler
			.load_location(self.region, node.destination);
		let source = self.data_handler.load_location(self.region, node.source);
		let size = self.data_handler.load(self.region, node.size);

		self.code_handler.emit_table_copy(destination, source, size);
	}

	fn handle_table_drop(&mut self, node: operation::TableDrop) {
		let source = self.data_handler.load(self.region, node.source);

		self.code_handler.emit_table_drop(source);
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
		let destination = self.data_handler.load_location(
			self.region,
			operation::Location {
				reference,
				..node.destination
			},
		);
		let source = self.data_handler.load(self.region, node.source);

		self.code_handler
			.emit_memory_store(destination, source, node.kind);
	}

	fn handle_memory_fill(&mut self, id: u32, node: operation::MemoryFill) {
		let reference = self.bridge(
			id,
			operation::MemoryFill::STATE_PORT,
			node.destination.reference,
		);
		let destination = self.data_handler.load_location(
			self.region,
			operation::Location {
				reference,
				..node.destination
			},
		);
		let byte = self.data_handler.load(self.region, node.byte);
		let size = self.data_handler.load(self.region, node.size);

		self.code_handler.emit_memory_fill(destination, byte, size);
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

		let destination = self.data_handler.load_location(
			self.region,
			operation::Location {
				reference: destination,
				..node.destination
			},
		);
		let source = self.data_handler.load_location(
			self.region,
			operation::Location {
				reference: source,
				..node.source
			},
		);
		let size = self.data_handler.load(self.region, node.size);

		self.code_handler
			.emit_memory_copy(destination, source, size);
	}

	fn handle_memory_drop(&mut self, id: u32, node: operation::MemoryDrop) {
		self.bridge(id, operation::MemoryDrop::STATE_PORT, node.source);
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
			| Node::RepeatArguments(_)
			| Node::Identity(_)
			| Node::Fence(_) => {}

			Node::RepeatResults(ref node) => self.handle_repeat_results(node),

			Node::Foreign(_) => unreachable!("the LuaJIT target consumes no foreign nodes"),

			Node::Import(_)
			| Node::Trap
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
			| Node::TableGet(_)
			| Node::TableSize(_)
			| Node::TableGrow(_)
			| Node::MemoryNew(_) => self.emit_expression(nodes, id),

			Node::Export(ref node) => self.handle_export(id, node),
			Node::Apply(ref node) => self.handle_call(id, node),
			Node::MutableGet(node) => self.handle_mutable_get(id, node),
			Node::MutableSet(node) => self.handle_mutable_set(id, node),
			Node::TableSet(node) => self.handle_table_set(node),
			Node::TableFill(node) => self.handle_table_fill(node),
			Node::TableCopy(node) => self.handle_table_copy(node),
			Node::TableDrop(node) => self.handle_table_drop(node),
			Node::MemoryLoad(node) => self.handle_memory_load(id, node),
			Node::MemoryStore(node) => self.handle_memory_store(id, node),
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
