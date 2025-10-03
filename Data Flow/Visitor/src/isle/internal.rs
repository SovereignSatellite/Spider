#![expect(
	clippy::match_ref_pats,
	clippy::pedantic,
	dead_code,
	non_snake_case,
	unused_imports,
	unused_variables
)]

use data_flow_graph::{
	Link,
	base::{IntegerBinaryOperator, IntegerType},
};

include!(concat!(env!("OUT_DIR"), "/isle.rs"));
