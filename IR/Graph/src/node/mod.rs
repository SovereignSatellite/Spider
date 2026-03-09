use alloc::boxed::Box;

use self::{
	control::{
		GammaIn, GammaOut, Import, LambdaIn, LambdaOut, OmegaIn, OmegaOut, RegionIn, RegionOut,
		ThetaIn, ThetaOut,
	},
	simple::{
		Apply, Fence, GlobalGet, GlobalNew, GlobalSet, Host, Identity, IntegerBinaryOperation,
		IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend, IntegerNarrow,
		IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden, MemoryCopy, MemoryDrop,
		MemoryFill, MemoryGrow, MemoryLoad, MemoryNew, MemorySize, MemoryStore,
		NumberBinaryOperation, NumberCompareOperation, NumberNarrow, NumberTransmuteToInteger,
		NumberTruncateToInteger, NumberUnaryOperation, NumberWiden, RefIsNull, TableCopy,
		TableDrop, TableFill, TableGet, TableGrow, TableNew, TableSet, TableSize,
	},
};

mod sinks;
mod sources;

pub mod control;
pub mod simple;

/// A node in the data flow graph.
#[derive(Default)]
pub enum Node {
	/// A lambda (function) input.
	LambdaIn(LambdaIn),
	/// A lambda (function) output.
	LambdaOut(LambdaOut),

	/// A region (scope) input.
	RegionIn(RegionIn),
	/// A region (scope) output.
	RegionOut(RegionOut),

	/// A gamma (conditional) input.
	GammaIn(GammaIn),
	/// A gamma (conditional) output.
	GammaOut(GammaOut),

	/// A theta (loop) input.
	ThetaIn(ThetaIn),
	/// A theta (loop) output.
	ThetaOut(ThetaOut),

	/// An omega (program) input.
	OmegaIn(OmegaIn),
	/// An omega (program) output.
	OmegaOut(OmegaOut),

	/// An external import.
	Import(Box<Import>),
	/// A host-defined operation.
	Host(Box<dyn Host>),

	/// An unreachable trap.
	#[default]
	Trap,
	/// A null reference constant.
	Null,

	/// A 32-bit integer constant.
	I32(i32),
	/// A 64-bit integer constant.
	I64(i64),
	/// A 32-bit float constant.
	F32(f32),
	/// A 64-bit float constant.
	F64(f64),

	/// An identity pass-through.
	Identity(Identity),
	/// An ordering fence.
	Fence(Fence),

	/// A function application.
	Apply(Apply),

	/// A reference null check.
	RefIsNull(RefIsNull),

	/// An integer unary operation.
	IntegerUnaryOperation(IntegerUnaryOperation),
	/// An integer binary operation.
	IntegerBinaryOperation(IntegerBinaryOperation),
	/// An integer comparison.
	IntegerCompareOperation(IntegerCompareOperation),
	/// An integer narrowing from 64-bit to 32-bit.
	IntegerNarrow(IntegerNarrow),
	/// An integer widening from 32-bit to 64-bit.
	IntegerWiden(IntegerWiden),
	/// An integer sign extension.
	IntegerExtend(IntegerExtend),
	/// An integer-to-floating-point conversion.
	IntegerConvertToNumber(IntegerConvertToNumber),
	/// An integer-to-floating-point bit reinterpretation.
	IntegerTransmuteToNumber(IntegerTransmuteToNumber),

	/// A floating-point unary operation.
	NumberUnaryOperation(NumberUnaryOperation),
	/// A floating-point binary operation.
	NumberBinaryOperation(NumberBinaryOperation),
	/// A floating-point comparison.
	NumberCompareOperation(NumberCompareOperation),
	/// A floating-point narrowing from 64-bit to 32-bit.
	NumberNarrow(NumberNarrow),
	/// A floating-point widening from 32-bit to 64-bit.
	NumberWiden(NumberWiden),
	/// A floating-point-to-integer truncation.
	NumberTruncateToInteger(NumberTruncateToInteger),
	/// A floating-point-to-integer bit reinterpretation.
	NumberTransmuteToInteger(NumberTransmuteToInteger),

	/// A global variable creation.
	GlobalNew(GlobalNew),
	/// A global variable read.
	GlobalGet(GlobalGet),
	/// A global variable write.
	GlobalSet(GlobalSet),

	/// A table creation.
	TableNew(TableNew),
	/// A table element read.
	TableGet(TableGet),
	/// A table element write.
	TableSet(TableSet),
	/// A table size query.
	TableSize(TableSize),
	/// A table grow.
	TableGrow(TableGrow),
	/// A table fill.
	TableFill(TableFill),
	/// A table copy.
	TableCopy(TableCopy),
	/// A table drop.
	TableDrop(TableDrop),

	/// A memory creation.
	MemoryNew(MemoryNew),
	/// A memory load.
	MemoryLoad(MemoryLoad),
	/// A memory store.
	MemoryStore(MemoryStore),
	/// A memory size query.
	MemorySize(MemorySize),
	/// A memory grow.
	MemoryGrow(MemoryGrow),
	/// A memory fill.
	MemoryFill(MemoryFill),
	/// A memory copy.
	MemoryCopy(MemoryCopy),
	/// A memory drop.
	MemoryDrop(MemoryDrop),
}

macro_rules! as_ref_inner {
	($inner:ident, $name:ident) => {
		#[doc = concat!("Returns a reference to the inner `", stringify!($inner), "`, if this node is one.")]
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
		#[doc = concat!("Returns a mutable reference to the inner `", stringify!($inner), "`, if this node is one.")]
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
