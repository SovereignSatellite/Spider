use luau_tree::expression::{Local, Name};

#[derive(Clone, Copy)]
pub struct Table {
	pub name: Name,
	pub len: u32,
}

#[derive(Clone, Copy)]
pub enum Place {
	Definition { name: Name },
	Assignment { name: Name },
	Overflow { table: Name, index: u16 },
}

impl Place {
	pub const fn into_definition(self) -> Name {
		if let Self::Definition { name } = self {
			name
		} else {
			panic!("should be a definition")
		}
	}

	pub const fn into_local(self) -> Local {
		match self {
			Self::Definition { name } | Self::Assignment { name } => Local::Fast { name },
			Self::Overflow { table, index } => Local::Slow { table, index },
		}
	}
}
