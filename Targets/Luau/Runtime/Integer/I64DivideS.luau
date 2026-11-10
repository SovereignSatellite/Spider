-- needs LuauBitsFromI64
-- needs LuauBitsIntoI64
-- needs I64DivisionU
-- needs I64NegateWords
const function i64_divide_s(lhs: number, rhs: number): number
	local low, high = luau_bits_from_i64(lhs)
	local divisor_low, divisor_high = luau_bits_from_i64(rhs)
	const negative = high >= 0x8000_0000
	const divisor_negative = divisor_high >= 0x8000_0000

	if negative then
		low, high = i64_negate_words(low, high)
	end

	if divisor_negative then
		divisor_low, divisor_high = i64_negate_words(divisor_low, divisor_high)
	end

	local quotient_low, quotient_high = i64_division_u(low, high, divisor_low, divisor_high)

	if negative ~= divisor_negative then
		quotient_low, quotient_high = i64_negate_words(quotient_low, quotient_high)
	end

	return (luau_bits_into_i64(quotient_low, quotient_high))
end
