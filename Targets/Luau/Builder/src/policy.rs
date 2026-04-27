use ir_allocator::Policy;
use ir_graph::Link;

pub const PHYSICAL_REGISTERS: u32 = 100;

static REGISTERS: [u8; PHYSICAL_REGISTERS as usize] = [0xFF; PHYSICAL_REGISTERS as usize];

pub struct LuauPolicy;

impl LuauPolicy {
	pub const fn new() -> Self {
		Self
	}
}

impl Policy for LuauPolicy {
	fn registers(&self) -> &[u8] {
		&REGISTERS
	}

	fn kind(&self, _scope: usize, _link: Link) -> u8 {
		0xFF
	}

	fn should_materialize(&self, _scope: usize, _link: Link) -> bool {
		true
	}
}
