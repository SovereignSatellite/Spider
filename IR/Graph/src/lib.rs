//! Data flow graph intermediate representation.

#![expect(
	clippy::multiple_inherent_impl,
	reason = "macro-generated visitor impl blocks are separate from constructor impl blocks"
)]

extern crate alloc;

pub use list;

pub use self::{
	link::Link,
	node::{Node, Region, control, simple},
};

mod link;
mod node;
