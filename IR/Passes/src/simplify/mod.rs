//! Graph-shrinking rewriters composed into the optimization fixpoint.

pub use self::common_node_eliminator::CommonNodeEliminator;

mod common_node_eliminator;

pub mod control_folder;
pub mod isle;
