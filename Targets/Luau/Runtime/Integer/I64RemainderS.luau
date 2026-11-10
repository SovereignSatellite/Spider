-- needs LuauBitsFromI64
-- needs LuauBitsIntoI64
-- needs I64DivisionU
-- needs I64NegateWords
const function i64_remainder_s(lhs: number, rhs: number): number
	local low, high = luau_bits_from_i64(lhs)
	local divisor_low, divisor_high = luau_bits_from_i64(rhs)
	const negative = high >= 0x8000_0000

	if negative then
		low, high = i64_negate_words(low, high)
	end

	if divisor_high >= 0x8000_0000 then
		divisor_low, divisor_high = i64_negate_words(divisor_low, divisor_high)
	end

	local _, _, remainder_low, remainder_high = i64_division_u(low, high, divisor_low, divisor_high)

	if negative then
		remainder_low, remainder_high = i64_negate_words(remainder_low, remainder_high)
	end

	return (luau_bits_into_i64(remainder_low, remainder_high))
end
