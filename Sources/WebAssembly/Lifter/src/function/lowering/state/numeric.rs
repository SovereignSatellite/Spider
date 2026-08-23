use ir_graph::{Link, Node, operation};
use web_assembly_control_flow::instruction::{
	IntegerBinaryOperation, IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend,
	IntegerNarrow, IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden,
	NumberBinaryOperation, NumberCompareOperation, NumberNarrow, NumberTransmuteToInteger,
	NumberTruncateToInteger, NumberUnaryOperation, NumberWiden,
};

use super::LoweringState;

impl LoweringState {
	pub fn handle_integer_unary_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: IntegerUnaryOperation,
	) {
		let IntegerUnaryOperation {
			destination,
			source,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = operation::integer::UnaryOperation::add_into(
			nodes,
			self.locals[usize::from(source)],
			kind,
			operator,
		);
	}

	pub fn handle_integer_binary_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: IntegerBinaryOperation,
	) {
		let IntegerBinaryOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = operation::integer::BinaryOperation::add_into(
			nodes,
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			kind,
			operator,
		);
	}

	pub fn handle_integer_compare_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: IntegerCompareOperation,
	) {
		let IntegerCompareOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = operation::integer::CompareOperation::add_into(
			nodes,
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			kind,
			operator,
		);
	}

	pub fn handle_integer_narrow(&mut self, nodes: &mut Vec<Node>, instruction: IntegerNarrow) {
		let IntegerNarrow {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			operation::IntegerNarrow::add_into(nodes, self.locals[usize::from(source)]);
	}

	pub fn handle_integer_widen(&mut self, nodes: &mut Vec<Node>, instruction: IntegerWiden) {
		let IntegerWiden {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			operation::IntegerWiden::add_into(nodes, self.locals[usize::from(source)]);
	}

	pub fn handle_integer_extend(&mut self, nodes: &mut Vec<Node>, instruction: IntegerExtend) {
		let IntegerExtend {
			destination,
			source,
			kind,
		} = instruction;

		self.locals[usize::from(destination)] =
			operation::IntegerSignExtend::add_into(nodes, self.locals[usize::from(source)], kind);
	}

	pub fn handle_integer_convert_to_number(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: IntegerConvertToNumber,
	) {
		let IntegerConvertToNumber {
			destination,
			source,
			is_signed,
			to,
			from,
		} = instruction;

		self.locals[usize::from(destination)] = operation::IntegerConvertToNumber::add_into(
			nodes,
			self.locals[usize::from(source)],
			is_signed,
			to,
			from,
		);
	}

	pub fn handle_integer_transmute_to_number(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: IntegerTransmuteToNumber,
	) {
		let IntegerTransmuteToNumber {
			destination,
			source,
			from,
		} = instruction;

		self.locals[usize::from(destination)] = operation::IntegerTransmuteToNumber::add_into(
			nodes,
			self.locals[usize::from(source)],
			from,
		);
	}

	pub fn handle_number_unary_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: NumberUnaryOperation,
	) {
		let NumberUnaryOperation {
			destination,
			source,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = operation::number::UnaryOperation::add_into(
			nodes,
			self.locals[usize::from(source)],
			kind,
			operator,
		);
	}

	pub fn handle_number_binary_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: NumberBinaryOperation,
	) {
		let NumberBinaryOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = operation::number::BinaryOperation::add_into(
			nodes,
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			kind,
			operator,
		);
	}

	pub fn handle_number_compare_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: NumberCompareOperation,
	) {
		let NumberCompareOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = operation::number::CompareOperation::add_into(
			nodes,
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			kind,
			operator,
		);
	}

	pub fn handle_number_narrow(&mut self, nodes: &mut Vec<Node>, instruction: NumberNarrow) {
		let NumberNarrow {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			operation::NumberNarrow::add_into(nodes, self.locals[usize::from(source)]);
	}

	pub fn handle_number_widen(&mut self, nodes: &mut Vec<Node>, instruction: NumberWiden) {
		let NumberWiden {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			operation::NumberWiden::add_into(nodes, self.locals[usize::from(source)]);
	}

	pub fn handle_number_truncate_to_integer(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: NumberTruncateToInteger,
	) {
		let NumberTruncateToInteger {
			destination,
			source,
			is_signed,
			is_saturating,
			to,
			from,
		} = instruction;

		self.locals[usize::from(destination)] = operation::NumberTruncateToInteger::add_into(
			nodes,
			self.locals[usize::from(source)],
			is_signed,
			is_saturating,
			to,
			from,
		);
	}

	pub fn handle_number_transmute_to_integer(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: NumberTransmuteToInteger,
	) {
		let NumberTransmuteToInteger {
			destination,
			source,
			from,
		} = instruction;

		self.locals[usize::from(destination)] = operation::NumberTransmuteToInteger::add_into(
			nodes,
			self.locals[usize::from(source)],
			from,
		);
	}

	pub fn handle_trapping_integer_binary_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: IntegerBinaryOperation,
	) {
		let IntegerBinaryOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		} = instruction;

		let result = operation::integer::BinaryOperation::add_into(
			nodes,
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			kind,
			operator,
		);
		let fence = operation::Fence::add_into(nodes, list::resizable![self.trap, result]);

		self.trap = Link(fence, 0);
		self.locals[usize::from(destination)] = result;
	}

	pub fn handle_trapping_number_truncate_to_integer(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: NumberTruncateToInteger,
	) {
		let NumberTruncateToInteger {
			destination,
			source,
			is_signed,
			is_saturating,
			to,
			from,
		} = instruction;

		let result = operation::NumberTruncateToInteger::add_into(
			nodes,
			self.locals[usize::from(source)],
			is_signed,
			is_saturating,
			to,
			from,
		);
		let fence = operation::Fence::add_into(nodes, list::resizable![self.trap, result]);

		self.trap = Link(fence, 0);
		self.locals[usize::from(destination)] = result;
	}
}
