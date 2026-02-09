-- SECTION count_ones_i64
-- NEEDS count_ones_i32
-- NEEDS from_bits_i64
-- NEEDS into_bits_i64
local function rt_count_ones_i64(source: number): number
	local source_1, source_2 = from_bits_i64(source)

	source_1 = rt_count_ones_i32(source_1)
	source_2 = rt_count_ones_i32(source_2)
	source = source_1 + source_2
	source = into_bits_i64(source, 0)

	return source
end

-- SECTION leading_zeroes_i64
-- NEEDS into_bits_i64
-- NEEDS raw_leading_zeroes_i64
local function rt_leading_zeroes_i64(source: number): number
	source = raw_leading_zeroes_i64(source)
	source = into_bits_i64(source, 0)

	return source
end

-- SECTION trailing_zeroes_i64
-- NEEDS into_bits_i64
-- NEEDS raw_trailing_zeroes_i64
local function rt_trailing_zeroes_i64(source: number): number
	source = raw_trailing_zeroes_i64(source)
	source = into_bits_i64(source, 0)

	return source
end

-- SECTION add_i64
-- NEEDS from_bits_i64
-- NEEDS into_bits_i64
local function rt_add_i64(lhs: number, rhs: number): number
	local lhs_1, lhs_2 = from_bits_i64(lhs)
	local rhs_1, rhs_2 = from_bits_i64(rhs)

	lhs = lhs_1 + rhs_1
	rhs = lhs_2 + rhs_2

	if lhs >= 0x1_0000_0000 then
		lhs = lhs - 0x1_0000_0000
		rhs = rhs + 1
	end

	if rhs >= 0x1_0000_0000 then
		rhs = rhs - 0x1_0000_0000
	end

	local result = into_bits_i64(lhs, rhs)

	return result
end

-- SECTION subtract_i64
-- NEEDS from_bits_i64
-- NEEDS into_bits_i64
local function rt_subtract_i64(lhs: number, rhs: number): number
	local lhs_1, lhs_2 = from_bits_i64(lhs)
	local rhs_1, rhs_2 = from_bits_i64(rhs)

	lhs = lhs_1 - rhs_1
	rhs = lhs_2 - rhs_2

	if lhs < 0 then
		lhs = lhs + 0x1_0000_0000
		rhs = rhs - 1
	end

	if rhs < 0 then
		rhs = rhs + 0x1_0000_0000
	end

	local result = into_bits_i64(lhs, rhs)

	return result
end

-- SECTION multiply_i64
-- NEEDS bit32_and
-- NEEDS bit32_replace
-- NEEDS bit32_rshift
-- NEEDS from_bits_i64_to_i16
-- NEEDS into_bits_i64
-- NEEDS multiply_fast_i64
local function rt_multiply_i64(lhs: number, rhs: number): number
	local result = multiply_fast_i64(lhs, rhs)

	if result then
		return result
	end

	local a00, a16, a32, a48 = from_bits_i64_to_i16(lhs)
	local b00, b16, b32, b48 = from_bits_i64_to_i16(rhs)

	local c00 = a00 * b00
	local c16 = bit32_rshift(c00, 16)

	c00 = bit32_and(c00, 0xFFFF)
	c16 = c16 + a16 * b00

	local c32 = bit32_rshift(c16, 16)

	c16 = bit32_and(c16, 0xFFFF)
	c16 = c16 + a00 * b16
	c32 = c32 + bit32_rshift(c16, 16)
	c16 = bit32_and(c16, 0xFFFF)
	c32 = c32 + a32 * b00

	local c48 = bit32_rshift(c32, 16)

	c32 = bit32_and(c32, 0xFFFF)
	c32 = c32 + a16 * b16
	c48 = c48 + bit32_rshift(c32, 16)
	c32 = bit32_and(c32, 0xFFFF)
	c32 = c32 + a00 * b32
	c48 = c48 + bit32_rshift(c32, 16)
	c32 = bit32_and(c32, 0xFFFF)
	c48 = c48 + a48 * b00 + a32 * b16 + a16 * b32 + a00 * b48
	c48 = bit32_and(c48, 0xFFFF)

	lhs = bit32_replace(c00, c16, 16, 16)
	rhs = bit32_replace(c32, c48, 16, 16)

	result = into_bits_i64(lhs, rhs)

	return result
end

-- SECTION divide_s64
-- NEEDS I64_8000_0000_0000_0000
-- NEEDS I64_FFFF_FFFF_FFFF_FFFF
-- NEEDS divide_u64
-- NEEDS equal_i64
-- NEEDS is_positive
-- NEEDS negate_i64
local function rt_divide_s64(lhs: number, rhs: number): number
	if rt_equal_i64(lhs, I64_8000_0000_0000_0000) and rt_equal_i64(rhs, I64_FFFF_FFFF_FFFF_FFFF) then
		error("integer overflow")
	end

	local should_negate = false

	if not is_positive(lhs) then
		lhs = negate_i64(lhs)
		should_negate = true
	end

	if not is_positive(rhs) then
		rhs = negate_i64(rhs)
		should_negate = not should_negate
	end

	local quotient = rt_divide_u64(lhs, rhs)

	if should_negate then
		quotient = negate_i64(quotient)
	end

	return quotient
end

-- SECTION divide_u64
-- NEEDS I64_0000_0000_0000_0001
-- NEEDS convert_u64_to_f64
-- NEEDS greater_than_equal_u64
-- NEEDS is_all_zeroes
-- NEEDS or_i64
-- NEEDS raw_leading_zeroes_i64
-- NEEDS raw_shift_left_i64
-- NEEDS subtract_i64
-- NEEDS truncate_f64_to_u64
local function rt_divide_u64(lhs: number, rhs: number): number
	if is_all_zeroes(rhs) then
		error("integer divide by zero")
	end

	local lhs_zeroes = raw_leading_zeroes_i64(lhs)
	local rhs_zeroes = raw_leading_zeroes_i64(rhs)

	if lhs_zeroes >= 11 and rhs_zeroes >= 11 then
		lhs = rt_convert_u64_to_f64(lhs)
		rhs = rt_convert_u64_to_f64(rhs)

		local result = rt_truncate_f64_to_u64(lhs / rhs)

		return result
	elseif lhs_zeroes > rhs_zeroes then
		return 0
	end

	local quotient = 0
	local remainder = lhs

	for offset = rhs_zeroes - lhs_zeroes, 0, -1 do
		local divisor = raw_shift_left_i64(rhs, offset)

		if rt_greater_than_equal_u64(remainder, divisor) then
			remainder = rt_subtract_i64(remainder, divisor)
			quotient = rt_or_i64(raw_shift_left_i64(I64_0000_0000_0000_0001, offset), quotient)
		end
	end

	return quotient
end

-- SECTION remainder_s64
-- NEEDS is_positive
-- NEEDS negate_i64
-- NEEDS remainder_u64
local function rt_remainder_s64(lhs: number, rhs: number): number
	local should_negate = false

	if not is_positive(lhs) then
		lhs = negate_i64(lhs)
		should_negate = true
	end

	if not is_positive(rhs) then
		rhs = negate_i64(rhs)
	end

	local remainder = rt_remainder_u64(lhs, rhs)

	if should_negate then
		remainder = negate_i64(remainder)
	end

	return remainder
end

-- SECTION remainder_u64
-- NEEDS convert_u64_to_f64
-- NEEDS greater_than_equal_u64
-- NEEDS is_all_zeroes
-- NEEDS raw_leading_zeroes_i64
-- NEEDS raw_shift_left_i64
-- NEEDS subtract_i64
-- NEEDS truncate_f64_to_u64
local function rt_remainder_u64(lhs: number, rhs: number): number
	if is_all_zeroes(rhs) then
		error("integer divide by zero")
	end

	local lhs_zeroes = raw_leading_zeroes_i64(lhs)
	local rhs_zeroes = raw_leading_zeroes_i64(rhs)

	if lhs_zeroes >= 11 and rhs_zeroes >= 11 then
		lhs = rt_convert_u64_to_f64(lhs)
		rhs = rt_convert_u64_to_f64(rhs)

		local result = rt_truncate_f64_to_u64(lhs % rhs)

		return result
	elseif lhs_zeroes > rhs_zeroes then
		return lhs
	end

	local remainder = lhs

	for offset = rhs_zeroes - lhs_zeroes, 0, -1 do
		local divisor = raw_shift_left_i64(rhs, offset)

		if rt_greater_than_equal_u64(remainder, divisor) then
			remainder = rt_subtract_i64(remainder, divisor)
		end
	end

	return remainder
end

-- SECTION and_i64
-- NEEDS bit32_and
-- NEEDS from_bits_i64
-- NEEDS into_bits_i64
local function rt_and_i64(lhs: number, rhs: number): number
	local lhs_1, lhs_2 = from_bits_i64(lhs)
	local rhs_1, rhs_2 = from_bits_i64(rhs)

	lhs = bit32_and(lhs_1, rhs_1)
	rhs = bit32_and(lhs_2, rhs_2)

	local result = into_bits_i64(lhs, rhs)

	return result
end

-- SECTION or_i64
-- NEEDS bit32_or
-- NEEDS from_bits_i64
-- NEEDS into_bits_i64
local function rt_or_i64(lhs: number, rhs: number): number
	local lhs_1, lhs_2 = from_bits_i64(lhs)
	local rhs_1, rhs_2 = from_bits_i64(rhs)

	lhs = bit32_or(lhs_1, rhs_1)
	rhs = bit32_or(lhs_2, rhs_2)

	local result = into_bits_i64(lhs, rhs)

	return result
end

-- SECTION exclusive_or_i64
-- NEEDS bit32_xor
-- NEEDS from_bits_i64
-- NEEDS into_bits_i64
local function rt_exclusive_or_i64(lhs: number, rhs: number): number
	local lhs_1, lhs_2 = from_bits_i64(lhs)
	local rhs_1, rhs_2 = from_bits_i64(rhs)

	lhs = bit32_xor(lhs_1, rhs_1)
	rhs = bit32_xor(lhs_2, rhs_2)

	local result = into_bits_i64(lhs, rhs)

	return result
end

-- SECTION shift_left_i64
-- NEEDS bit32_and
-- NEEDS from_bits_i64
-- NEEDS raw_shift_left_i64
local function rt_shift_left_i64(lhs: number, rhs: number): number
	rhs = bit32_and(from_bits_i64(rhs), 0x3F)
	lhs = raw_shift_left_i64(lhs, rhs)

	return lhs
end

-- SECTION shift_right_s64
-- NEEDS bit32_and
-- NEEDS from_bits_i64
-- NEEDS raw_shift_right_s64
local function rt_shift_right_s64(lhs: number, rhs: number): number
	rhs = bit32_and(from_bits_i64(rhs), 0x3F)
	lhs = raw_shift_right_s64(lhs, rhs)

	return lhs
end

-- SECTION shift_right_u64
-- NEEDS bit32_and
-- NEEDS from_bits_i64
-- NEEDS raw_shift_right_u64
local function rt_shift_right_u64(lhs: number, rhs: number): number
	rhs = bit32_and(from_bits_i64(rhs), 0x3F)
	lhs = raw_shift_right_u64(lhs, rhs)

	return lhs
end

-- SECTION rotate_left_i64
-- NEEDS I64_0000_0000_0000_0040
-- NEEDS or_i64
-- NEEDS shift_left_i64
-- NEEDS shift_right_u64
-- NEEDS subtract_i64
local function rt_rotate_left_i64(lhs: number, rhs: number): number
	local result = rt_shift_left_i64(lhs, rhs)

	rhs = rt_subtract_i64(I64_0000_0000_0000_0040, rhs)
	lhs = rt_shift_right_u64(lhs, rhs)

	result = rt_or_i64(lhs, result)

	return result
end

-- SECTION rotate_right_i64
-- NEEDS I64_0000_0000_0000_0040
-- NEEDS or_i64
-- NEEDS shift_left_i64
-- NEEDS shift_right_u64
-- NEEDS subtract_i64
local function rt_rotate_right_i64(lhs: number, rhs: number): number
	local result = rt_shift_right_u64(lhs, rhs)

	rhs = rt_subtract_i64(I64_0000_0000_0000_0040, rhs)
	lhs = rt_shift_left_i64(lhs, rhs)

	result = rt_or_i64(lhs, result)

	return result
end

-- SECTION equal_i64
-- NEEDS from_bits_i64
local function rt_equal_i64(lhs: number, rhs: number): boolean
	local lhs_1, lhs_2 = from_bits_i64(lhs)
	local rhs_1, rhs_2 = from_bits_i64(rhs)

	return lhs_1 == rhs_1 and lhs_2 == rhs_2
end

-- SECTION not_equal_i64
-- NEEDS from_bits_i64
local function rt_not_equal_i64(lhs: number, rhs: number): boolean
	local lhs_1, lhs_2 = from_bits_i64(lhs)
	local rhs_1, rhs_2 = from_bits_i64(rhs)

	return lhs_1 ~= rhs_1 or lhs_2 ~= rhs_2
end

-- SECTION less_than_s64
-- NEEDS flip_most_significant
-- NEEDS less_than_u64
local function rt_less_than_s64(lhs: number, rhs: number): boolean
	lhs = flip_most_significant(lhs)
	rhs = flip_most_significant(rhs)

	local result = rt_less_than_u64(lhs, rhs)

	return result
end

-- SECTION less_than_u64
-- NEEDS from_bits_i64
local function rt_less_than_u64(lhs: number, rhs: number): boolean
	local lhs_1, lhs_2 = from_bits_i64(lhs)
	local rhs_1, rhs_2 = from_bits_i64(rhs)

	return lhs_2 < rhs_2 or (lhs_2 == rhs_2 and lhs_1 < rhs_1)
end

-- SECTION greater_than_s64
-- NEEDS flip_most_significant
-- NEEDS greater_than_u64
local function rt_greater_than_s64(lhs: number, rhs: number): boolean
	lhs = flip_most_significant(lhs)
	rhs = flip_most_significant(rhs)

	local result = rt_greater_than_u64(lhs, rhs)

	return result
end

-- SECTION greater_than_u64
-- NEEDS from_bits_i64
local function rt_greater_than_u64(lhs: number, rhs: number): boolean
	local lhs_1, lhs_2 = from_bits_i64(lhs)
	local rhs_1, rhs_2 = from_bits_i64(rhs)

	return lhs_2 > rhs_2 or (lhs_2 == rhs_2 and lhs_1 > rhs_1)
end

-- SECTION less_than_equal_s64
-- NEEDS greater_than_s64
local function rt_less_than_equal_s64(lhs: number, rhs: number): boolean
	return not rt_greater_than_s64(lhs, rhs)
end

-- SECTION less_than_equal_u64
-- NEEDS greater_than_u64
local function rt_less_than_equal_u64(lhs: number, rhs: number): boolean
	return not rt_greater_than_u64(lhs, rhs)
end

-- SECTION greater_than_equal_s64
-- NEEDS less_than_s64
local function rt_greater_than_equal_s64(lhs: number, rhs: number): boolean
	return not rt_less_than_s64(lhs, rhs)
end

-- SECTION greater_than_equal_u64
-- NEEDS less_than_u64
local function rt_greater_than_equal_u64(lhs: number, rhs: number): boolean
	return not rt_less_than_u64(lhs, rhs)
end

-- SECTION narrow_i64
-- NEEDS from_bits_i64
local function rt_narrow_i64(source: number): number
	source = from_bits_i64(source)

	return source
end

-- SECTION extend_s8_to_i64
-- NEEDS bit32_and
-- NEEDS bit32_or
-- NEEDS from_bits_i64
-- NEEDS into_bits_i64
local function rt_extend_s8_to_i64(source: number): number
	source = from_bits_i64(source)
	source = bit32_and(source, 0xFF)

	if source < 0x80 then
		source = into_bits_i64(source, 0)
	else
		source = bit32_or(source - 0x100, 0)
		source = into_bits_i64(source, 0xFFFF_FFFF)
	end

	return source
end

-- SECTION extend_s16_to_i64
-- NEEDS bit32_and
-- NEEDS bit32_or
-- NEEDS from_bits_i64
-- NEEDS into_bits_i64
local function rt_extend_s16_to_i64(source: number): number
	source = from_bits_i64(source)
	source = bit32_and(source, 0xFFFF)

	if source < 0x8000 then
		source = into_bits_i64(source, 0)
	else
		source = bit32_or(source - 0x1_0000, 0)
		source = into_bits_i64(source, 0xFFFF_FFFF)
	end

	return source
end

-- SECTION extend_s32_to_i64
-- NEEDS bit32_and
-- NEEDS bit32_or
-- NEEDS from_bits_i64
-- NEEDS into_bits_i64
local function rt_extend_s32_to_i64(source: number): number
	source = from_bits_i64(source)
	source = bit32_and(source, 0xFFFF_FFFF)

	if source < 0x8000_0000 then
		source = into_bits_i64(source, 0)
	else
		source = bit32_or(source - 0x1_0000_0000, 0)
		source = into_bits_i64(source, 0xFFFF_FFFF)
	end

	return source
end

-- SECTION convert_s64_to_f32
-- NEEDS convert_u64_to_f32
-- NEEDS is_positive
-- NEEDS negate_f32
-- NEEDS negate_i64
local function rt_convert_s64_to_f32(source: number): number
	if is_positive(source) then
		source = rt_convert_u64_to_f32(source)
	else
		source = negate_i64(source)
		source = rt_convert_u64_to_f32(source)
		source = rt_negate_f32(source)
	end

	return source
end

-- SECTION convert_u64_to_f32
-- NEEDS convert_u64_to_f64
-- NEEDS narrow_f64
local function rt_convert_u64_to_f32(source: number): number
	source = rt_convert_u64_to_f64(source)
	source = rt_narrow_f64(source)

	return source
end

-- SECTION convert_s64_to_f64
-- NEEDS convert_u64_to_f64
-- NEEDS is_positive
-- NEEDS negate_i64
local function rt_convert_s64_to_f64(source: number): number
	if is_positive(source) then
		source = rt_convert_u64_to_f64(source)
	else
		source = negate_i64(source)
		source = rt_convert_u64_to_f64(source)
		source = -source
	end

	return source
end

-- SECTION convert_u64_to_f64
-- NEEDS from_bits_i64
local function rt_convert_u64_to_f64(source: number): number
	local source_1, source_2 = from_bits_i64(source)

	return source_1 + source_2 * 0x1_0000_0000
end

-- SECTION transmute_i64_to_f64
local function rt_transmute_i64_to_f64(source: number): number
	return source
end
