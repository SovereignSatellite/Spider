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

fn fmt_instruction(instruction: Instruction, f: &mut Formatter<'_>) -> Result {
	use core::fmt::Debug;

	match instruction {
		Instruction::LocalSet(instruction) => Debug::fmt(&instruction, f),
		Instruction::LocalBranch(instruction) => Debug::fmt(&instruction, f),
		Instruction::I32Constant(instruction) => Debug::fmt(&instruction, f),
		Instruction::I64Constant(instruction) => Debug::fmt(&instruction, f),
		Instruction::F32Constant(instruction) => Debug::fmt(&instruction, f),
		Instruction::F64Constant(instruction) => Debug::fmt(&instruction, f),
		Instruction::RefIsNull(instruction) => Debug::fmt(&instruction, f),
		Instruction::RefNull(instruction) => Debug::fmt(&instruction, f),
		Instruction::RefFunction(instruction) => Debug::fmt(&instruction, f),
		Instruction::Call(instruction) => Debug::fmt(&instruction, f),
		Instruction::Unreachable => write!(f, "Unreachable"),
		Instruction::IntegerUnaryOperation(instruction) => Debug::fmt(&instruction, f),
		Instruction::IntegerBinaryOperation(instruction) => Debug::fmt(&instruction, f),
		Instruction::IntegerCompareOperation(instruction) => Debug::fmt(&instruction, f),
		Instruction::IntegerNarrow(instruction) => Debug::fmt(&instruction, f),
		Instruction::IntegerWiden(instruction) => Debug::fmt(&instruction, f),
		Instruction::IntegerExtend(instruction) => Debug::fmt(&instruction, f),
		Instruction::IntegerConvertToNumber(instruction) => Debug::fmt(&instruction, f),
		Instruction::IntegerTransmuteToNumber(instruction) => Debug::fmt(&instruction, f),
		Instruction::NumberUnaryOperation(instruction) => Debug::fmt(&instruction, f),
		Instruction::NumberBinaryOperation(instruction) => Debug::fmt(&instruction, f),
		Instruction::NumberCompareOperation(instruction) => Debug::fmt(&instruction, f),
		Instruction::NumberNarrow(instruction) => Debug::fmt(&instruction, f),
		Instruction::NumberWiden(instruction) => Debug::fmt(&instruction, f),
		Instruction::NumberTruncateToInteger(instruction) => Debug::fmt(&instruction, f),
		Instruction::NumberTransmuteToInteger(instruction) => Debug::fmt(&instruction, f),
		Instruction::GlobalGet(instruction) => Debug::fmt(&instruction, f),
		Instruction::GlobalSet(instruction) => Debug::fmt(&instruction, f),
		Instruction::TableGet(instruction) => Debug::fmt(&instruction, f),
		Instruction::TableSet(instruction) => Debug::fmt(&instruction, f),
		Instruction::TableSize(instruction) => Debug::fmt(&instruction, f),
		Instruction::TableGrow(instruction) => Debug::fmt(&instruction, f),
		Instruction::TableFill(instruction) => Debug::fmt(&instruction, f),
		Instruction::TableCopy(instruction) => Debug::fmt(&instruction, f),
		Instruction::TableInit(instruction) => Debug::fmt(&instruction, f),
		Instruction::ElementsDrop(instruction) => Debug::fmt(&instruction, f),
		Instruction::MemoryLoad(instruction) => Debug::fmt(&instruction, f),
		Instruction::MemoryStore(instruction) => Debug::fmt(&instruction, f),
		Instruction::MemorySize(instruction) => Debug::fmt(&instruction, f),
		Instruction::MemoryGrow(instruction) => Debug::fmt(&instruction, f),
		Instruction::MemoryFill(instruction) => Debug::fmt(&instruction, f),
		Instruction::MemoryCopy(instruction) => Debug::fmt(&instruction, f),
		Instruction::MemoryInit(instruction) => Debug::fmt(&instruction, f),
		Instruction::DataDrop(instruction) => Debug::fmt(&instruction, f),
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

	fn fmt_nodes(&self, f: &mut Formatter<'_>) -> Result {
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

	fn fmt_edges(&self, f: &mut Formatter<'_>) -> Result {
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
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		writeln!(f, "digraph {{")?;

		self.fmt_nodes(f)?;
		self.fmt_edges(f)?;

		writeln!(f, "}}")
	}
}
