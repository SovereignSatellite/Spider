-- needs BitsScratch
local function lua_jit_bits_from_f32(value)
	BITS_SCRATCH.f32 = value

	return BITS_SCRATCH.i32
end
