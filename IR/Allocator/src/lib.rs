//! Register allocation for IR regions.

#![no_std]

extern crate alloc;

pub use self::{allocator::Allocator, policy::Policy};

mod allocator;
mod coloring;
mod lifetime;
mod policy;
