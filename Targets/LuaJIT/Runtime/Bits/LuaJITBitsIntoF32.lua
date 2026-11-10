-- needs BitsScratch
local function lua_jit_bits_into_f32(bits)
	BITS_SCRATCH.i32 = bits

	return BITS_SCRATCH.f32
end
