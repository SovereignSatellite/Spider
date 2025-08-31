use alloc::vec::Vec;
use data_flow_graph::{
	mvp::{
		Call, GlobalGet, GlobalSet, MemoryCopy, MemoryFill, MemoryGrow, MemoryInit, MemoryLoad,
		MemorySize, MemoryStore, TableCopy, TableFill, TableGet, TableGrow, TableInit, TableSet,
		TableSize,
	},
	DataFlowGraph, Link, Node,
};
use hashbrown::HashMap;

fn has_local_result(node: &Node, id: u32, port: u16, locals: &[u32]) -> bool {
	match node {
		Node::LambdaIn(_)
		| Node::RegionIn(_)
		| Node::RegionOut(_)
		| Node::GammaIn(_)
		| Node::GammaOut(_)
		| Node::ThetaIn(_)
		| Node::ThetaOut(_)
		| Node::OmegaIn(_)
		| Node::OmegaOut(_)
		| Node::GlobalNew(_)
		| Node::TableNew(_)
		| Node::ElementsNew(_)
		| Node::MemoryNew(_)
		| Node::DataNew(_) => true,

		Node::LambdaOut(_)
		| Node::Import(_)
		| Node::Trap
		| Node::Null
		| Node::Identity(_)
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
		| Node::IntegerExtend(_)
		| Node::IntegerConvertToNumber(_)
		| Node::IntegerTransmuteToNumber(_)
		| Node::NumberUnaryOperation(_)
		| Node::NumberBinaryOperation(_)
		| Node::NumberCompareOperation(_)
		| Node::NumberNarrow(_)
		| Node::NumberWiden(_)
		| Node::NumberTruncateToInteger(_)
		| Node::NumberTransmuteToInteger(_) => locals.binary_search(&id).is_ok(),

		Node::Host(host) => panic!("unknown host operation {id} `{}`", host.identifier()),

		Node::Call(call) => port < call.results && locals.binary_search(&id).is_ok(),
		Node::Merge(_)
		| Node::GlobalSet(_)
		| Node::TableSet(_)
		| Node::TableFill(_)
		| Node::TableCopy(_)
		| Node::TableInit(_)
		| Node::ElementsDrop(_)
		| Node::MemoryStore(_)
		| Node::MemoryFill(_)
		| Node::MemoryCopy(_)
		| Node::MemoryInit(_)
		| Node::DataDrop(_) => false,

		Node::GlobalGet(_) => port == GlobalGet::RESULT_PORT && locals.binary_search(&id).is_ok(),
		Node::TableGet(_) => port == TableGet::RESULT_PORT && locals.binary_search(&id).is_ok(),
		Node::TableSize(_) => port == TableSize::RESULT_PORT && locals.binary_search(&id).is_ok(),
		Node::TableGrow(_) => port == TableGrow::RESULT_PORT && locals.binary_search(&id).is_ok(),
		Node::MemoryLoad(_) => port == MemoryLoad::RESULT_PORT && locals.binary_search(&id).is_ok(),
		Node::MemorySize(_) => port == MemorySize::RESULT_PORT && locals.binary_search(&id).is_ok(),
		Node::MemoryGrow(_) => port == MemoryGrow::RESULT_PORT && locals.binary_search(&id).is_ok(),
	}
}

fn set_max_lifetime(link: Link, until: u32, lifetimes: &mut HashMap<Link, u32>) {
	let last = lifetimes.entry(link).or_default();

	*last = until.max(*last);
}

pub struct LifetimeFinder {
	stack: Vec<Link>,
}

impl LifetimeFinder {
	pub const fn new() -> Self {
		Self { stack: Vec::new() }
	}

	#[expect(clippy::too_many_lines)]
	fn push_producers(&mut self, node: &Node, id: u32, locals: &[u32]) {
		match *node {
			Node::LambdaIn(_)
			| Node::LambdaOut(_)
			| Node::RegionIn(_)
			| Node::RegionOut(_)
			| Node::GammaIn(_)
			| Node::GammaOut(_)
			| Node::ThetaIn(_)
			| Node::ThetaOut(_)
			| Node::OmegaIn(_)
			| Node::OmegaOut(_) => node.for_each_argument(|link| self.stack.push(link)),

			Node::Trap
			| Node::Null
			| Node::I32(_)
			| Node::I64(_)
			| Node::F32(_)
			| Node::F64(_)
			| Node::Merge(_)
			| Node::GlobalGet(_)
			| Node::TableSize(_)
			| Node::ElementsDrop(_)
			| Node::MemorySize(_)
			| Node::DataNew(_)
			| Node::DataDrop(_) => {}

			Node::Import(_)
			| Node::Identity(_)
			| Node::RefIsNull(_)
			| Node::IntegerUnaryOperation(_)
			| Node::IntegerBinaryOperation(_)
			| Node::IntegerCompareOperation(_)
			| Node::IntegerNarrow(_)
			| Node::IntegerWiden(_)
			| Node::IntegerExtend(_)
			| Node::IntegerConvertToNumber(_)
			| Node::IntegerTransmuteToNumber(_)
			| Node::NumberUnaryOperation(_)
			| Node::NumberBinaryOperation(_)
			| Node::NumberCompareOperation(_)
			| Node::NumberNarrow(_)
			| Node::NumberWiden(_)
			| Node::NumberTruncateToInteger(_)
			| Node::NumberTransmuteToInteger(_)
			| Node::GlobalNew(_)
			| Node::TableNew(_)
			| Node::ElementsNew(_)
			| Node::MemoryNew(_) => {
				if locals.binary_search(&id).is_ok() {
					node.for_each_argument(|link| self.stack.push(link));
				}
			}

			Node::Host(ref host) => panic!("unknown host operation {id} `{}`", host.identifier()),

			Node::Call(Call {
				ref arguments,
				function,
				states,
				..
			}) => {
				if locals.binary_search(&id).is_ok() {
					let last = arguments.len() - usize::from(states);

					self.stack.push(function);
					self.stack.extend_from_slice(&arguments[..last]);
				}
			}

			Node::GlobalSet(GlobalSet { source, .. }) => self.stack.push(source),
			Node::TableGet(TableGet { source }) | Node::MemoryLoad(MemoryLoad { source, .. }) => {
				if locals.binary_search(&id).is_ok() {
					self.stack.push(source.offset);
				}
			}
			Node::TableSet(TableSet {
				destination,
				source,
			})
			| Node::MemoryStore(MemoryStore {
				destination,
				source,
				..
			}) => {
				self.stack.push(destination.offset);
				self.stack.push(source);
			}
			Node::TableGrow(TableGrow {
				initializer, size, ..
			}) => {
				if locals.binary_search(&id).is_ok() {
					self.stack.push(initializer);
					self.stack.push(size);
				}
			}
			Node::TableFill(TableFill {
				destination,
				source,
				size,
			}) => {
				self.stack.push(destination.offset);
				self.stack.push(source);
				self.stack.push(size);
			}
			Node::TableCopy(TableCopy {
				destination,
				source,
				size,
			})
			| Node::MemoryCopy(MemoryCopy {
				destination,
				source,
				size,
			})
			| Node::TableInit(TableInit {
				destination,
				source,
				size,
				..
			})
			| Node::MemoryInit(MemoryInit {
				destination,
				source,
				size,
				..
			}) => {
				self.stack.push(destination.offset);
				self.stack.push(source.offset);
				self.stack.push(size);
			}
			Node::MemoryGrow(MemoryGrow { size, .. }) => {
				if locals.binary_search(&id).is_ok() {
					self.stack.push(size);
				}
			}
			Node::MemoryFill(MemoryFill {
				destination,
				byte,
				size,
			}) => {
				self.stack.push(destination.offset);
				self.stack.push(byte);
				self.stack.push(size);
			}
		}
	}

	#[expect(clippy::too_many_lines)]
	fn push_continuations(&mut self, node: &Node, port: u16) {
		match *node {
			Node::LambdaIn(_)
			| Node::LambdaOut(_)
			| Node::RegionIn(_)
			| Node::RegionOut(_)
			| Node::GammaIn(_)
			| Node::GammaOut(_)
			| Node::ThetaIn(_)
			| Node::ThetaOut(_)
			| Node::OmegaIn(_)
			| Node::OmegaOut(_)
			| Node::Trap
			| Node::Null
			| Node::I32(_)
			| Node::I64(_)
			| Node::F32(_)
			| Node::F64(_)
			| Node::GlobalNew(_)
			| Node::GlobalGet(_)
			| Node::TableNew(_)
			| Node::ElementsNew(_)
			| Node::MemoryNew(_)
			| Node::DataNew(_) => {}

			Node::Import(_)
			| Node::Identity(_)
			| Node::Merge(_)
			| Node::RefIsNull(_)
			| Node::IntegerUnaryOperation(_)
			| Node::IntegerBinaryOperation(_)
			| Node::IntegerCompareOperation(_)
			| Node::IntegerNarrow(_)
			| Node::IntegerWiden(_)
			| Node::IntegerExtend(_)
			| Node::IntegerConvertToNumber(_)
			| Node::IntegerTransmuteToNumber(_)
			| Node::NumberUnaryOperation(_)
			| Node::NumberBinaryOperation(_)
			| Node::NumberCompareOperation(_)
			| Node::NumberNarrow(_)
			| Node::NumberWiden(_)
			| Node::NumberTruncateToInteger(_)
			| Node::NumberTransmuteToInteger(_)
			| Node::ElementsDrop(_)
			| Node::DataDrop(_) => node.for_each_argument(|link| self.stack.push(link)),

			Node::Host(ref host) => panic!("unknown host operation `{}`", host.identifier()),

			Node::Call(Call {
				ref arguments,
				results,
				states,
				..
			}) => {
				if let Some(result) = port.checked_sub(results) {
					let position = arguments.len() - usize::from(states) + usize::from(result);

					self.stack.push(arguments[position]);
				} else {
					let end = arguments.len() - usize::from(states);

					self.stack.extend_from_slice(&arguments[..end]);
				}
			}

			Node::GlobalSet(GlobalSet { destination, .. }) => self.stack.push(destination),
			Node::TableGet(TableGet { source }) => {
				if port == TableGet::RESULT_PORT {
					self.stack.push(source.offset);
				} else if port == TableGet::STATE_PORT {
					self.stack.push(source.reference);
				}
			}
			Node::TableSet(TableSet { destination, .. }) => self.stack.push(destination.reference),
			Node::TableSize(TableSize { source }) => {
				if port == TableSize::STATE_PORT {
					self.stack.push(source);
				}
			}
			Node::TableGrow(TableGrow {
				destination,
				initializer,
				size,
			}) => {
				if port == TableGrow::RESULT_PORT {
					self.stack.push(initializer);
					self.stack.push(size);
				} else if port == TableGrow::STATE_PORT {
					self.stack.push(destination);
				}
			}
			Node::TableFill(TableFill { destination, .. })
			| Node::MemoryStore(MemoryStore { destination, .. })
			| Node::MemoryFill(MemoryFill { destination, .. }) => {
				self.stack.push(destination.reference);
			}
			Node::TableCopy(TableCopy {
				destination,
				source,
				..
			}) => {
				if port == TableCopy::DESTINATION_STATE_PORT {
					self.stack.push(destination.reference);
				} else if port == TableCopy::SOURCE_STATE_PORT {
					self.stack.push(source.reference);
				}
			}
			Node::TableInit(TableInit {
				destination,
				source,
				..
			}) => {
				if port == TableInit::DESTINATION_STATE_PORT {
					self.stack.push(destination.reference);
				} else if port == TableInit::SOURCE_STATE_PORT {
					self.stack.push(source.reference);
				}
			}
			Node::MemoryLoad(MemoryLoad { source, .. }) => {
				if port == MemoryLoad::RESULT_PORT {
					self.stack.push(source.offset);
				} else if port == MemoryLoad::STATE_PORT {
					self.stack.push(source.reference);
				}
			}
			Node::MemorySize(MemorySize { source }) => {
				if port == MemorySize::STATE_PORT {
					self.stack.push(source);
				}
			}
			Node::MemoryGrow(MemoryGrow { destination, size }) => {
				if port == MemoryGrow::RESULT_PORT {
					self.stack.push(size);
				} else if port == MemoryGrow::STATE_PORT {
					self.stack.push(destination);
				}
			}
			Node::MemoryCopy(MemoryCopy {
				destination,
				source,
				..
			}) => {
				if port == MemoryCopy::DESTINATION_STATE_PORT {
					self.stack.push(destination.reference);
				} else if port == MemoryCopy::SOURCE_STATE_PORT {
					self.stack.push(source.reference);
				}
			}
			Node::MemoryInit(MemoryInit {
				destination,
				source,
				..
			}) => {
				if port == MemoryInit::DESTINATION_STATE_PORT {
					self.stack.push(destination.reference);
				} else if port == MemoryInit::SOURCE_STATE_PORT {
					self.stack.push(source.reference);
				}
			}
		}
	}

	fn pop_producers(
		&mut self,
		lifetimes: &mut HashMap<Link, u32>,
		graph: &DataFlowGraph,
		until: u32,
		locals: &[u32],
	) {
		while let Some(link) = self.stack.pop() {
			let node = graph.get(link.0);

			if has_local_result(node, link.0, link.1, locals) {
				set_max_lifetime(link, until, lifetimes);
			} else {
				self.push_continuations(node, link.1);
			}
		}
	}

	pub fn run(
		&mut self,
		lifetimes: &mut HashMap<Link, u32>,
		graph: &DataFlowGraph,
		locals: &[u32],
	) {
		lifetimes.clear();

		for (node, id) in graph.nodes().zip(0..) {
			self.push_producers(node, id, locals);

			self.stack.sort_unstable();
			self.stack.dedup();

			self.pop_producers(lifetimes, graph, id, locals);
		}
	}
}
