-- needs BitsScratch
local function lua_jit_bits_into_f64(bits)
	BITS_SCRATCH.i64 = bits

	return BITS_SCRATCH.f64
end
