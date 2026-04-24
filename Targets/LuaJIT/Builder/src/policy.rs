use ir_allocator::Policy;
use ir_graph::Link;

pub const PHYSICAL_REGISTERS: u32 = 197;

static REGISTERS: [u8; PHYSICAL_REGISTERS as usize] = [0xFF; PHYSICAL_REGISTERS as usize];

pub struct LuaJITPolicy;

impl LuaJITPolicy {
	pub const fn new() -> Self {
		Self
	}
}

impl Policy for LuaJITPolicy {
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
