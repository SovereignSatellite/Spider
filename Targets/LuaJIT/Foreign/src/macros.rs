//! Declaration macros for `LuaJIT` foreign operation nodes.

/// Declares a unary `LuaJIT` foreign operation taking a single source link.
macro_rules! define_unary_operation {
	($name:ident, $identifier:literal) => {
		#[doc = concat!("The `", $identifier, "` `LuaJIT` foreign operation.")]
		#[derive(Clone, Copy)]
		pub struct $name {
			/// The source operand link.
			pub source: Link,
		}

		impl $name {
			/// The number of output ports.
			pub const RESULT_COUNT: u16 = 1;
			/// The result port index.
			pub const RESULT_PORT: u16 = 0;

			/// Adds the operation to the graph and returns its result link.
			#[must_use = "inserted nodes without live consumers are dead"]
			pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
				let Ok(id) = nodes.len().try_into() else {
					unreachable!()
				};

				nodes.push(Node::Foreign(Box::new(Self { source })));

				Link(id, Self::RESULT_PORT)
			}
		}

		impl Foreign for $name {
			fn identifier(&self) -> &'static str {
				$identifier
			}

			fn result_count(&self) -> u16 {
				Self::RESULT_COUNT
			}

			fn duplicate(&self) -> Box<dyn Foreign> {
				Box::new(*self)
			}

			fn for_each_outer(&self, handler: &mut dyn FnMut(Link)) {
				handler(self.source);
			}

			fn for_each_mut_outer(&mut self, handler: &mut dyn FnMut(&mut Link)) {
				handler(&mut self.source);
			}
		}
	};
}

/// Declares a binary `LuaJIT` foreign operation taking a left- and right-hand link.
macro_rules! define_binary_operation {
	($name:ident, $identifier:literal) => {
		#[doc = concat!("The `", $identifier, "` `LuaJIT` foreign operation.")]
		#[derive(Clone, Copy)]
		pub struct $name {
			/// The left-hand operand link.
			pub lhs: Link,
			/// The right-hand operand link.
			pub rhs: Link,
		}

		impl $name {
			/// The number of output ports.
			pub const RESULT_COUNT: u16 = 1;
			/// The result port index.
			pub const RESULT_PORT: u16 = 0;

			/// Adds the operation to the graph and returns its result link.
			#[must_use = "inserted nodes without live consumers are dead"]
			pub fn add_into(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
				let Ok(id) = nodes.len().try_into() else {
					unreachable!()
				};

				nodes.push(Node::Foreign(Box::new(Self { lhs, rhs })));

				Link(id, Self::RESULT_PORT)
			}

			/// Adds the operation with a constant right-hand operand.
			#[must_use = "inserted nodes without live consumers are dead"]
			pub fn add_fast_into(nodes: &mut Vec<Node>, lhs: Link, rhs: u32) -> Link {
				let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
				let rhs = Node::add_i32_into(nodes, rhs);

				Self::add_into(nodes, lhs, rhs)
			}
		}

		impl Foreign for $name {
			fn identifier(&self) -> &'static str {
				$identifier
			}

			fn result_count(&self) -> u16 {
				Self::RESULT_COUNT
			}

			fn duplicate(&self) -> Box<dyn Foreign> {
				Box::new(*self)
			}

			fn for_each_outer(&self, handler: &mut dyn FnMut(Link)) {
				handler(self.lhs);
				handler(self.rhs);
			}

			fn for_each_mut_outer(&mut self, handler: &mut dyn FnMut(&mut Link)) {
				handler(&mut self.lhs);
				handler(&mut self.rhs);
			}
		}
	};
}
