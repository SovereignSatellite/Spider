-- needs LuauBit32Bor
-- needs LuauBit32Lshift
const function i64_division_u(
	low: number,
	high: number,
	divisor_low: number,
	divisor_high: number
): (number, number, number, number)
	const value = low + high * 0x1_0000_0000
	const divisor = divisor_low + divisor_high * 0x1_0000_0000
	local quotient_low, quotient_high, remainder_low, remainder_high

	if high < 0x20_0000 then
		const quotient = value // divisor
		const remainder = value - quotient * divisor
		quotient_low, quotient_high = quotient % 0x1_0000_0000, quotient // 0x1_0000_0000
		remainder_low, remainder_high = remainder % 0x1_0000_0000, remainder // 0x1_0000_0000
	elseif divisor_high < 0x20 then
		-- A divisor below 2^37 takes digits of 32, 16 and 16 bits: a remainder times 2^16 plus a digit stays below 2^53.
		quotient_high = high // divisor
		const upper_digit = low // 0x1_0000
		const upper = (high - quotient_high * divisor) * 0x1_0000 + upper_digit
		const quotient_upper = upper // divisor
		const lower = (upper - quotient_upper * divisor) * 0x1_0000 + (low - upper_digit * 0x1_0000)
		const quotient_lower = lower // divisor
		const remainder = lower - quotient_lower * divisor
		quotient_low = quotient_upper * 0x1_0000 + quotient_lower
		remainder_low, remainder_high = remainder % 0x1_0000_0000, remainder // 0x1_0000_0000
	else
		-- A quotient below 2^27 scaled down past its rounding error is exact or one low, so its product does not pass the
		-- value. The product's rounded form is within 2^12 of the exact one, so its high word is the nearest integer.
		quotient_low, quotient_high = value * 0.999999999999999 // divisor, 0
		const divisor_upper = divisor_low // 0x1_0000
		const product_low = luau_bit32_bor(
			quotient_low * (divisor_low - divisor_upper * 0x1_0000) + luau_bit32_lshift(quotient_low * divisor_upper, 16),
			0
		)
		local difference = low - product_low
		remainder_low = difference % 0x1_0000_0000
		remainder_high = high + difference // 0x1_0000_0000 - ((quotient_low * divisor - product_low) * (1 / 0x1_0000_0000) + 0.5) // 1

		if remainder_high > divisor_high or (remainder_high == divisor_high and remainder_low >= divisor_low) then
			quotient_low += 1
			difference = remainder_low - divisor_low
			remainder_low = difference % 0x1_0000_0000
			remainder_high += difference // 0x1_0000_0000 - divisor_high
		end
	end

	return quotient_low, quotient_high, remainder_low, remainder_high
end
