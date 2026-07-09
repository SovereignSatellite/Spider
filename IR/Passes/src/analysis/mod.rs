//! Read-only graph queries built once per region and consulted by passes.

pub use self::successor_finder::{Successor, SuccessorFinder};

mod successor_finder;
