-- needs BitsScratch
-- needs LuauBufferReadU32
-- needs LuauBufferWriteF64
const function luau_bits_from_i64(value: number): (number, number)
	luau_buffer_write_f64(BITS_SCRATCH, 0, value)

	return luau_buffer_read_u32(BITS_SCRATCH, 0), (luau_buffer_read_u32(BITS_SCRATCH, 4))
end
