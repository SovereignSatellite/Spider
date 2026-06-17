//! Data flow graph intermediate representation.

extern crate alloc;

pub use list;

pub use self::{
	link::Link,
	node::{Node, Region, Shape, foreign, operation, region},
};

mod link;
mod node;

pub mod region_driver;
pub mod tracer;
