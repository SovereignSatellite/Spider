use alloc::boxed::Box;

use super::{
	mvp::{
		Call, DataDrop, DataNew, ElementsDrop, ElementsNew, GlobalGet, GlobalNew, GlobalSet, Host,
		Identity, IntegerBinaryOperation, IntegerCompareOperation, IntegerConvertToNumber,
		IntegerExtend, IntegerNarrow, IntegerTransmuteToNumber, IntegerUnaryOperation,
		IntegerWiden, MemoryCopy, MemoryFill, MemoryGrow, MemoryInit, MemoryLoad, MemoryNew,
		MemorySize, MemoryStore, Merge, NumberBinaryOperation, NumberCompareOperation,
		NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger, NumberUnaryOperation,
		NumberWiden, RefIsNull, TableCopy, TableFill, TableGet, TableGrow, TableInit, TableNew,
		TableSet, TableSize,
	},
	nested::{
		GammaIn, GammaOut, Import, LambdaIn, LambdaOut, OmegaIn, OmegaOut, RegionIn, RegionOut,
		ThetaIn, ThetaOut,
	},
};

pub enum Node {
	LambdaIn(LambdaIn),
	LambdaOut(LambdaOut),

	RegionIn(RegionIn),
	RegionOut(RegionOut),

	GammaIn(GammaIn),
	GammaOut(GammaOut),

	ThetaIn(ThetaIn),
	ThetaOut(ThetaOut),

	OmegaIn(OmegaIn),
	OmegaOut(OmegaOut),

	Import(Box<Import>),
	Host(Box<dyn Host>),

	Trap,
	Null,

	Identity(Identity),

	I32(i32),
	I64(i64),
	F32(f32),
	F64(f64),

	Call(Call),
	Merge(Merge),

	RefIsNull(RefIsNull),

	IntegerUnaryOperation(IntegerUnaryOperation),
	IntegerBinaryOperation(IntegerBinaryOperation),
	IntegerCompareOperation(IntegerCompareOperation),
	IntegerNarrow(IntegerNarrow),
	IntegerWiden(IntegerWiden),
	IntegerExtend(IntegerExtend),
	IntegerConvertToNumber(IntegerConvertToNumber),
	IntegerTransmuteToNumber(IntegerTransmuteToNumber),

	NumberUnaryOperation(NumberUnaryOperation),
	NumberBinaryOperation(NumberBinaryOperation),
	NumberCompareOperation(NumberCompareOperation),
	NumberNarrow(NumberNarrow),
	NumberWiden(NumberWiden),
	NumberTruncateToInteger(NumberTruncateToInteger),
	NumberTransmuteToInteger(NumberTransmuteToInteger),

	GlobalNew(GlobalNew),
	GlobalGet(GlobalGet),
	GlobalSet(GlobalSet),

	TableNew(TableNew),
	TableGet(TableGet),
	TableSet(TableSet),
	TableSize(TableSize),
	TableGrow(TableGrow),
	TableFill(TableFill),
	TableCopy(TableCopy),
	TableInit(TableInit),

	ElementsNew(ElementsNew),
	ElementsDrop(ElementsDrop),

	MemoryNew(MemoryNew),
	MemoryLoad(MemoryLoad),
	MemoryStore(MemoryStore),
	MemorySize(MemorySize),
	MemoryGrow(MemoryGrow),
	MemoryFill(MemoryFill),
	MemoryCopy(MemoryCopy),
	MemoryInit(MemoryInit),

	DataNew(DataNew),
	DataDrop(DataDrop),
}

macro_rules! as_ref_inner {
	($inner:ident, $name:ident) => {
		#[must_use]
		pub const fn $name(&self) -> Option<&$inner> {
			if let Self::$inner(node) = self {
				Some(node)
			} else {
				None
			}
		}
	};
}

macro_rules! as_mut_inner {
	($inner:ident, $name:ident) => {
		#[must_use]
		pub const fn $name(&mut self) -> Option<&mut $inner> {
			if let Self::$inner(node) = self {
				Some(node)
			} else {
				None
			}
		}
	};
}

impl Node {
	as_ref_inner!(LambdaIn, as_lambda_in);
	as_ref_inner!(LambdaOut, as_lambda_out);
	as_ref_inner!(RegionIn, as_region_in);
	as_ref_inner!(RegionOut, as_region_out);
	as_ref_inner!(GammaIn, as_gamma_in);
	as_ref_inner!(GammaOut, as_gamma_out);
	as_ref_inner!(ThetaIn, as_theta_in);
	as_ref_inner!(ThetaOut, as_theta_out);
	as_ref_inner!(OmegaIn, as_omega_in);
	as_ref_inner!(OmegaOut, as_omega_out);

	as_mut_inner!(LambdaIn, as_mut_lambda_in);
	as_mut_inner!(LambdaOut, as_mut_lambda_out);
	as_mut_inner!(RegionIn, as_mut_region_in);
	as_mut_inner!(RegionOut, as_mut_region_out);
	as_mut_inner!(GammaIn, as_mut_gamma_in);
	as_mut_inner!(GammaOut, as_mut_gamma_out);
	as_mut_inner!(ThetaIn, as_mut_theta_in);
	as_mut_inner!(ThetaOut, as_mut_theta_out);
	as_mut_inner!(OmegaIn, as_mut_omega_in);
	as_mut_inner!(OmegaOut, as_mut_omega_out);
}

impl Default for Node {
	fn default() -> Self {
		Self::Trap
	}
}
