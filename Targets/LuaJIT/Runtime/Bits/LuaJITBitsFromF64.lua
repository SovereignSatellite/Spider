-- needs BitsScratch
local function lua_jit_bits_from_f64(value)
	BITS_SCRATCH.f64 = value

	return BITS_SCRATCH.i64
end
