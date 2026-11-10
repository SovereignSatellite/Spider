-- needs BitsScratch
-- needs LuauBufferReadF32
-- needs LuauBufferWriteF32
const function luau_bits_round_f32(value: number): number
	luau_buffer_write_f32(BITS_SCRATCH, 0, value)

	return (luau_buffer_read_f32(BITS_SCRATCH, 0))
end
