use core::fmt::{Display, Formatter, Result};

use crate::{ControlFlowGraph, Instruction};

#[derive(PartialEq, Eq, Clone, Copy)]
enum Vertex {
	Assignment,
	Selection,
	Empty,
	Instructions,
}

impl Vertex {
	const fn from_instructions(instructions: &[Instruction]) -> Self {
		match instructions {
			[Instruction::I32Constant(_)] => Self::Assignment,
			[Instruction::LocalBranch(_)] => Self::Selection,
			[] => Self::Empty,
			_ => Self::Instructions,
		}
	}

	const fn group(self) -> &'static str {
		match self {
			Self::Assignment => "A",
			Self::Selection => "B",
			Self::Empty => "C",
			Self::Instructions => "D",
		}
	}

	const fn color(self) -> &'static str {
		match self {
			Self::Assignment | Self::Selection => "#EF8784",
			Self::Empty => "#C2C5FA",
			Self::Instructions => "#FBE78E",
		}
	}
}

impl Display for Vertex {
	fn fmt(&self, f: &mut Formatter) -> Result {
		writeln!(
			f,
			"\tnode [fillcolor = \"{}\", group = {}];",
			self.color(),
			self.group()
		)
	}
}

fn fmt_instruction(instruction: Instruction, f: &mut Formatter) -> Result {
	use core::fmt::Debug;

	match instruction {
		Instruction::LocalSet(local_set) => Debug::fmt(&local_set, f),
		Instruction::LocalBranch(local_branch) => Debug::fmt(&local_branch, f),
		Instruction::I32Constant(i32_constant) => Debug::fmt(&i32_constant, f),
		Instruction::I64Constant(i64_constant) => Debug::fmt(&i64_constant, f),
		Instruction::F32Constant(f32_constant) => Debug::fmt(&f32_constant, f),
		Instruction::F64Constant(f64_constant) => Debug::fmt(&f64_constant, f),
		Instruction::RefIsNull(ref_is_null) => Debug::fmt(&ref_is_null, f),
		Instruction::RefNull(ref_null) => Debug::fmt(&ref_null, f),
		Instruction::RefFunction(ref_function) => Debug::fmt(&ref_function, f),
		Instruction::Call(call) => Debug::fmt(&call, f),
		Instruction::Unreachable => write!(f, "Unreachable"),
		Instruction::IntegerUnaryOperation(integer_unary_operation) => {
			Debug::fmt(&integer_unary_operation, f)
		}
		Instruction::IntegerBinaryOperation(integer_binary_operation) => {
			Debug::fmt(&integer_binary_operation, f)
		}
		Instruction::IntegerCompareOperation(integer_compare_operation) => {
			Debug::fmt(&integer_compare_operation, f)
		}
		Instruction::IntegerNarrow(integer_narrow) => Debug::fmt(&integer_narrow, f),
		Instruction::IntegerWiden(integer_widen) => Debug::fmt(&integer_widen, f),
		Instruction::IntegerExtend(integer_extend) => Debug::fmt(&integer_extend, f),
		Instruction::IntegerConvertToNumber(integer_convert_to_number) => {
			Debug::fmt(&integer_convert_to_number, f)
		}
		Instruction::IntegerTransmuteToNumber(integer_transmute_to_number) => {
			Debug::fmt(&integer_transmute_to_number, f)
		}
		Instruction::NumberUnaryOperation(number_unary_operation) => {
			Debug::fmt(&number_unary_operation, f)
		}
		Instruction::NumberBinaryOperation(number_binary_operation) => {
			Debug::fmt(&number_binary_operation, f)
		}
		Instruction::NumberCompareOperation(number_compare_operation) => {
			Debug::fmt(&number_compare_operation, f)
		}
		Instruction::NumberNarrow(number_narrow) => Debug::fmt(&number_narrow, f),
		Instruction::NumberWiden(number_widen) => Debug::fmt(&number_widen, f),
		Instruction::NumberTruncateToInteger(number_truncate_to_integer) => {
			Debug::fmt(&number_truncate_to_integer, f)
		}
		Instruction::NumberTransmuteToInteger(number_transmute_to_integer) => {
			Debug::fmt(&number_transmute_to_integer, f)
		}
		Instruction::GlobalGet(global_get) => Debug::fmt(&global_get, f),
		Instruction::GlobalSet(global_set) => Debug::fmt(&global_set, f),
		Instruction::TableGet(table_get) => Debug::fmt(&table_get, f),
		Instruction::TableSet(table_set) => Debug::fmt(&table_set, f),
		Instruction::TableSize(table_size) => Debug::fmt(&table_size, f),
		Instruction::TableGrow(table_grow) => Debug::fmt(&table_grow, f),
		Instruction::TableFill(table_fill) => Debug::fmt(&table_fill, f),
		Instruction::TableCopy(table_copy) => Debug::fmt(&table_copy, f),
		Instruction::TableInit(table_init) => Debug::fmt(&table_init, f),
		Instruction::ElementsDrop(elements_drop) => Debug::fmt(&elements_drop, f),
		Instruction::MemoryLoad(memory_load) => Debug::fmt(&memory_load, f),
		Instruction::MemoryStore(memory_store) => Debug::fmt(&memory_store, f),
		Instruction::MemorySize(memory_size) => Debug::fmt(&memory_size, f),
		Instruction::MemoryGrow(memory_grow) => Debug::fmt(&memory_grow, f),
		Instruction::MemoryFill(memory_fill) => Debug::fmt(&memory_fill, f),
		Instruction::MemoryCopy(memory_copy) => Debug::fmt(&memory_copy, f),
		Instruction::MemoryInit(memory_init) => Debug::fmt(&memory_init, f),
		Instruction::DataDrop(data_drop) => Debug::fmt(&data_drop, f),
	}
}

pub struct Dot<'inner> {
	inner: &'inner ControlFlowGraph,
}

impl<'inner> Dot<'inner> {
	#[must_use]
	pub const fn new(inner: &'inner ControlFlowGraph) -> Self {
		Self { inner }
	}

	fn fmt_nodes(&self, f: &mut Formatter) -> Result {
		writeln!(f, "\tnode [shape = box, style = filled, ordering = out];")?;

		let mut last_vertex = Vertex::Instructions;

		last_vertex.fmt(f)?;

		self.inner.block_ids().try_for_each(|id| {
			let instructions = self.inner.instructions(id);
			let vertex = Vertex::from_instructions(instructions);

			if vertex != last_vertex {
				last_vertex = vertex;

				last_vertex.fmt(f)?;
			}

			write!(f, "\tN{id} [xlabel = {id}, label = \"")?;

			instructions.iter().try_for_each(|&instruction| {
				fmt_instruction(instruction, f)?;

				write!(f, "\\l")
			})?;

			writeln!(f, "\"];")
		})
	}

	fn fmt_edges(&self, f: &mut Formatter) -> Result {
		writeln!(f, "\tedge [color = \"#444477\"];")?;

		self.inner.block_ids().try_for_each(|id| {
			self.inner.successors(id).try_for_each(|successor| {
				let style = if successor <= id {
					" [style = dashed]"
				} else {
					""
				};

				writeln!(f, "\tN{id} -> N{successor}{style};")
			})
		})
	}
}

impl Display for Dot<'_> {
	fn fmt(&self, f: &mut Formatter) -> Result {
		writeln!(f, "digraph {{")?;

		self.fmt_nodes(f)?;
		self.fmt_edges(f)?;

		writeln!(f, "}}")
	}
}
