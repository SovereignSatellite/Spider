-- needs BitsScratch
-- needs LuauBufferReadF32
-- needs LuauBufferWriteU32
const function luau_bits_into_f32(bits: number): number
	luau_buffer_write_u32(BITS_SCRATCH, 0, bits)

	return (luau_buffer_read_f32(BITS_SCRATCH, 0))
end
