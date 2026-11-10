-- needs BitsScratch
-- needs LuauBufferReadF64
-- needs LuauBufferWriteU32
const function luau_bits_into_i64(low: number, high: number): number
	luau_buffer_write_u32(BITS_SCRATCH, 0, low)
	luau_buffer_write_u32(BITS_SCRATCH, 4, high)

	return (luau_buffer_read_f64(BITS_SCRATCH, 0))
end
