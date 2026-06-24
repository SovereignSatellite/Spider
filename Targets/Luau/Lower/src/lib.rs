//! Luau-specific lowering of trivial IR operations into foreign-node trees.
//!
//! Runs as the injected pass inside the optimizer fixpoint: each trivial operation is
//! expanded in place into a tree of [`luau_foreign`] nodes, and the generic passes then
//! compact and simplify the result.

pub use self::dispatch::apply;

mod convert;
mod dispatch;
mod f32;
mod f64;
mod i32;
mod i64;
mod replace;
mod round;
