//! Lowers trivial IR operations into Luau foreign-node trees.
//! Runs inside the optimizer fixpoint so generic passes simplify each expansion.

pub use self::dispatch::apply;

mod boolean;
mod convert;
mod dispatch;
mod f32;
mod f64;
mod i32;
mod i64;
mod memory;
mod replace;
mod round;
mod table;
mod truncate;
