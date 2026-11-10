-- needs LuauBitsFromI64
-- needs LuauBitsIntoI64
-- needs LuauBit32Bxor
-- needs I64NegateWords
-- needs I64ConvertUToF32
const function i64_convert_s_to_f32(value: number): number
	const low, high = luau_bits_from_i64(value)

	if high < 0x8000_0000 then
		return i64_convert_u_to_f32(value)
	end

	return luau_bit32_bxor(i64_convert_u_to_f32(luau_bits_into_i64(i64_negate_words(low, high))), 0x8000_0000)
end
