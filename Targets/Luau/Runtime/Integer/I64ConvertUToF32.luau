-- needs LuauBitsFromI64
-- needs LuauBitsFromF32
-- needs LuauBit32Band
-- needs LuauBit32Bor
-- needs LuauBit32Lshift
-- needs LuauBit32Countlz
const function i64_convert_u_to_f32(value: number): number
	local low, high = luau_bits_from_i64(value)

	if high >= 0x20_0000 then
		const dropped = luau_bit32_lshift(1, 11 - luau_bit32_countlz(high)) - 1
		low = luau_bit32_band(luau_bit32_bor(low, luau_bit32_band(low, dropped) + dropped), 0xFFFF_FFFF - dropped)
	end

	return luau_bits_from_f32(low + high * 0x1_0000_0000)
end
