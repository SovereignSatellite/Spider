pub use self::{
	turing_machine::lift as from_turing_machine, web_assembly::lift as from_web_assembly,
};

mod turing_machine;
mod web_assembly;
