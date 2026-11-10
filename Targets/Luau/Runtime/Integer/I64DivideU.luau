-- needs LuauBitsFromI64
-- needs LuauBitsIntoI64
-- needs I64DivisionU
const function i64_divide_u(lhs: number, rhs: number): number
	const low, high = luau_bits_from_i64(lhs)
	const divisor_low, divisor_high = luau_bits_from_i64(rhs)
	const quotient_low, quotient_high = i64_division_u(low, high, divisor_low, divisor_high)

	return (luau_bits_into_i64(quotient_low, quotient_high))
end
