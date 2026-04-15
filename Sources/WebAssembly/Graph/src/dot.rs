use core::fmt::{Display, Formatter, Result};

use super::{ControlFlowGraph, Instruction};

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
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		writeln!(
			f,
			"\tnode [fillcolor = \"{}\", group = {}];",
			self.color(),
			self.group()
		)
	}
}

fn fmt_instruction(instruction: Instruction, formatter: &mut Formatter<'_>) -> Result {
	use core::fmt::Debug;

	match instruction {
		Instruction::LocalSet(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::LocalBranch(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::I32Constant(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::I64Constant(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::F32Constant(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::F64Constant(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::RefIsNull(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::RefNull(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::RefFunction(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::Call(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::Unreachable => write!(formatter, "Unreachable"),
		Instruction::IntegerUnaryOperation(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::IntegerBinaryOperation(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::IntegerCompareOperation(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::IntegerNarrow(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::IntegerWiden(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::IntegerExtend(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::IntegerConvertToNumber(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::IntegerTransmuteToNumber(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::NumberUnaryOperation(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::NumberBinaryOperation(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::NumberCompareOperation(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::NumberNarrow(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::NumberWiden(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::NumberTruncateToInteger(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::NumberTransmuteToInteger(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::GlobalGet(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::GlobalSet(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::TableGet(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::TableSet(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::TableSize(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::TableGrow(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::TableFill(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::TableCopy(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::TableInit(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::ElementsDrop(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::MemoryLoad(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::MemoryStore(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::MemorySize(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::MemoryGrow(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::MemoryFill(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::MemoryCopy(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::MemoryInit(instruction) => Debug::fmt(&instruction, formatter),
		Instruction::DataDrop(instruction) => Debug::fmt(&instruction, formatter),
	}
}

/// A DOT graph formatter for control flow graphs.
pub struct Dot<'inner> {
	inner: &'inner ControlFlowGraph,
}

impl<'inner> Dot<'inner> {
	/// Creates a new DOT formatter for the given control flow graph.
	#[must_use]
	pub const fn new(inner: &'inner ControlFlowGraph) -> Self {
		Self { inner }
	}

	fn fmt_nodes(&self, formatter: &mut Formatter<'_>) -> Result {
		writeln!(
			formatter,
			"\tnode [shape = box, style = filled, ordering = out];"
		)?;

		let mut last_vertex = Vertex::Instructions;

		last_vertex.fmt(formatter)?;

		self.inner.block_ids().try_for_each(|id| {
			let instructions = self.inner.instructions(id);
			let vertex = Vertex::from_instructions(instructions);

			if vertex != last_vertex {
				last_vertex = vertex;

				last_vertex.fmt(formatter)?;
			}

			write!(formatter, "\tN{id} [xlabel = {id}, label = \"")?;

			instructions.iter().try_for_each(|&instruction| {
				fmt_instruction(instruction, formatter)?;

				write!(formatter, "\\l")
			})?;

			writeln!(formatter, "\"];")
		})
	}

	fn fmt_edges(&self, formatter: &mut Formatter<'_>) -> Result {
		writeln!(formatter, "\tedge [color = \"#444477\"];")?;

		self.inner.block_ids().try_for_each(|id| {
			self.inner.successors(id).try_for_each(|successor| {
				let style = if successor <= id {
					" [style = dashed]"
				} else {
					""
				};

				writeln!(formatter, "\tN{id} -> N{successor}{style};")
			})
		})
	}
}

impl Display for Dot<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		writeln!(f, "digraph {{")?;

		self.fmt_nodes(f)?;
		self.fmt_edges(f)?;

		writeln!(f, "}}")
	}
}
