//! Data flow graph intermediate representation.

extern crate alloc;

pub use list;

pub use self::{
	link::Link,
	node::{Node, Region, foreign, operation, region},
};

mod link;
mod node;
pub mod region_driver;
