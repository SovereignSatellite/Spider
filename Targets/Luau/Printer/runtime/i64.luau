-- SECTION create_i64_from_u32
-- NEEDS buffer_read_f64
-- NEEDS buffer_write_u32
-- NEEDS raw_memory
local function rt_create_i64_from_u32(source_1: number, source_2: number): number
	buffer_write_u32(RAW_MEMORY, 0, source_1)
	buffer_write_u32(RAW_MEMORY, 4, source_2)

	return buffer_read_f64(RAW_MEMORY, 0)
end

-- SECTION create_u32_from_i64
-- NEEDS buffer_read_u32
-- NEEDS buffer_write_f64
-- NEEDS raw_memory
local function rt_create_u32_from_i64(source: number): (number, number)
	buffer_write_f64(RAW_MEMORY, 0, source)

	return buffer_read_u32(RAW_MEMORY, 0), buffer_read_u32(RAW_MEMORY, 4)
end

-- SECTION create_u32_from_i64_1
-- NEEDS buffer_read_u32
-- NEEDS buffer_write_f64
-- NEEDS raw_memory
local function rt_create_u32_from_i64_1(source: number): number
	buffer_write_f64(RAW_MEMORY, 0, source)

	return buffer_read_u32(RAW_MEMORY, 0)
end

-- SECTION create_u32_from_i64_2
-- NEEDS buffer_read_u32
-- NEEDS buffer_write_f64
-- NEEDS raw_memory
local function rt_create_u32_from_i64_2(source: number): number
	buffer_write_f64(RAW_MEMORY, 0, source)

	return buffer_read_u32(RAW_MEMORY, 4)
end

-- SECTION create_u16_from_i64
-- NEEDS bit_and
-- NEEDS bit_rshift
-- NEEDS create_u32_from_i64
local function rt_create_u16_from_i64(source: number): (number, number, number, number)
	local source_1, source_2 = rt_create_u32_from_i64(source)

	return bit_and(source_1, 0xFFFF), bit_rshift(source_1, 16), bit_and(source_2, 0xFFFF), bit_rshift(source_2, 16)
end

-- SECTION constant_zero_u64
local ZERO_U64 = rt_create_i64_from_u32(0, 0)

-- SECTION constant_one_u64
local ONE_U64 = rt_create_i64_from_u32(1, 0)

-- SECTION constant_all_u64
local ALL_U64 = rt_create_i64_from_u32(0xFFFF_FFFF, 0xFFFF_FFFF)

-- SECTION constant_minimum_s64
local MINIMUM_S64 = rt_create_i64_from_u32(0x0000_0000, 0x8000_0000)

-- SECTION constant_maximum_s64
local MAXIMUM_S64 = rt_create_i64_from_u32(0xFFFF_FFFF, 0x7FFF_FFFF)

-- SECTION count_ones_i64
-- NEEDS count_ones_i32
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
local function rt_count_ones_i64(source: number): number
	local source_1, source_2 = rt_create_u32_from_i64(source)
	local ones = rt_count_ones_i32(source_1) + rt_count_ones_i32(source_2)

	return rt_create_i64_from_u32(ones, 0)
end

-- SECTION raw_leading_zeroes_i64
-- NEEDS bit_countlz
-- NEEDS create_u32_from_i64
local function rt_raw_leading_zeroes_i64(source: number): number
	local source_1, source_2 = rt_create_u32_from_i64(source)

	return if source_2 == 0 then bit_countlz(source_1) + 32 else bit_countlz(source_2)
end

-- SECTION leading_zeroes_i64
-- NEEDS create_i64_from_u32
-- NEEDS raw_leading_zeroes_i64
local function rt_leading_zeroes_i64(source: number): number
	local zeroes = rt_raw_leading_zeroes_i64(source)

	return rt_create_i64_from_u32(zeroes, 0)
end

-- SECTION trailing_zeroes_i64
-- NEEDS bit_countrz
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
local function rt_trailing_zeroes_i64(source: number): number
	local source_1, source_2 = rt_create_u32_from_i64(source)
	local zeroes = if source_1 == 0 then bit_countrz(source_2) + 32 else bit_countrz(source_1)

	return rt_create_i64_from_u32(zeroes, 0)
end

-- SECTION negate_i64
-- NEEDS bit_not
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
local function rt_negate_i64(source: number): number
	local source_1, source_2 = rt_create_u32_from_i64(source)
	local result_1 = bit_not(source_1) + 1
	local result_2 = bit_not(source_2)

	if result_1 >= 0x1_0000_0000 then
		result_1 = result_1 - 0x1_0000_0000
		result_2 = result_2 + 1
	end

	if result_2 >= 0x1_0000_0000 then
		result_2 = result_2 - 0x1_0000_0000
	end

	return rt_create_i64_from_u32(result_1, result_2)
end

-- SECTION add_i64
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
local function rt_add_i64(lhs: number, rhs: number): number
	local lhs_1, lhs_2 = rt_create_u32_from_i64(lhs)
	local rhs_1, rhs_2 = rt_create_u32_from_i64(rhs)

	local result_1 = lhs_1 + rhs_1
	local result_2 = lhs_2 + rhs_2

	if result_1 >= 0x1_0000_0000 then
		result_1 = result_1 - 0x1_0000_0000
		result_2 = result_2 + 1
	end

	if result_2 >= 0x1_0000_0000 then
		result_2 = result_2 - 0x1_0000_0000
	end

	return rt_create_i64_from_u32(result_1, result_2)
end

-- SECTION subtract_i64
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
local function rt_subtract_i64(lhs: number, rhs: number): number
	local lhs_1, lhs_2 = rt_create_u32_from_i64(lhs)
	local rhs_1, rhs_2 = rt_create_u32_from_i64(rhs)

	local result_1 = lhs_1 - rhs_1
	local result_2 = lhs_2 - rhs_2

	if result_1 < 0 then
		result_1 = result_1 + 0x1_0000_0000
		result_2 = result_2 - 1
	end

	if result_2 < 0 then
		result_2 = result_2 + 0x1_0000_0000
	end

	return rt_create_i64_from_u32(result_1, result_2)
end

-- SECTION multiply_fast_i64
-- NEEDS create_u32_from_i64
-- NEEDS truncate_f64_to_u64
local function rt_multiply_fast_i64(lhs: number, rhs: number): number?
	local lhs_1, lhs_2 = rt_create_u32_from_i64(lhs)
	local rhs_1, rhs_2 = rt_create_u32_from_i64(rhs)

	return if lhs_2 == 0
			and rhs_2 == 0
			and lhs_1 <= 0x400_0000
			and rhs_1 <= 0x400_0000
		then rt_truncate_f64_to_u64(lhs_1 * rhs_1)
		else nil
end

-- SECTION multiply_i64
-- NEEDS bit_and
-- NEEDS bit_replace
-- NEEDS bit_rshift
-- NEEDS create_i64_from_u32
-- NEEDS create_u16_from_i64
-- NEEDS multiply_fast_i64
local function rt_multiply_i64(lhs: number, rhs: number): number
	local result = rt_multiply_fast_i64(lhs, rhs)

	if result then
		return result
	end

	local a00, a16, a32, a48 = rt_create_u16_from_i64(lhs)
	local b00, b16, b32, b48 = rt_create_u16_from_i64(rhs)

	local c00 = a00 * b00
	local c16 = bit_rshift(c00, 16)

	c00 = bit_and(c00, 0xFFFF)
	c16 = c16 + a16 * b00

	local c32 = bit_rshift(c16, 16)

	c16 = bit_and(c16, 0xFFFF)
	c16 = c16 + a00 * b16
	c32 = c32 + bit_rshift(c16, 16)
	c16 = bit_and(c16, 0xFFFF)
	c32 = c32 + a32 * b00

	local c48 = bit_rshift(c32, 16)

	c32 = bit_and(c32, 0xFFFF)
	c32 = c32 + a16 * b16
	c48 = c48 + bit_rshift(c32, 16)
	c32 = bit_and(c32, 0xFFFF)
	c32 = c32 + a00 * b32
	c48 = c48 + bit_rshift(c32, 16)
	c32 = bit_and(c32, 0xFFFF)
	c48 = c48 + a48 * b00 + a32 * b16 + a16 * b32 + a00 * b48
	c48 = bit_and(c48, 0xFFFF)

	local result_1 = bit_replace(c00, c16, 16, 16)
	local result_2 = bit_replace(c32, c48, 16, 16)

	return rt_create_i64_from_u32(result_1, result_2)
end

-- SECTION is_zero_i64
-- NEEDS create_u32_from_i64_2
local function rt_is_zero_i64(source: number): boolean
	if source == 0 then
		local source_2 = rt_create_u32_from_i64_2(source)

		return source_2 == 0
	end

	return false
end

-- SECTION divide_s64
-- NEEDS constant_all_u64
-- NEEDS constant_minimum_s64
-- NEEDS divide_u64
-- NEEDS is_positive
-- NEEDS negate_i64
local function rt_divide_s64(lhs: number, rhs: number): number
	if rt_equal_i64(lhs, MINIMUM_S64) == 1 and rt_equal_i64(rhs, ALL_U64) == 1 then
		error("integer overflow", 2)
	end

	local lhs_is_positive = is_positive(lhs)
	local rhs_is_positive = is_positive(rhs)

	if not lhs_is_positive then
		lhs = rt_negate_i64(lhs)
	end

	if not rhs_is_positive then
		rhs = rt_negate_i64(rhs)
	end

	local quotient = rt_divide_u64(lhs, rhs)

	if lhs_is_positive ~= rhs_is_positive then
		quotient = rt_negate_i64(quotient)
	end

	return quotient
end

-- SECTION divide_u64
-- NEEDS constant_one_u64
-- NEEDS convert_u64_to_f64
-- NEEDS create_i64_from_u32
-- NEEDS greater_than_equal_u64
-- NEEDS is_zero_i64
-- NEEDS or_i64
-- NEEDS raw_leading_zeroes_i64
-- NEEDS shift_left_i64
-- NEEDS subtract_i64
-- NEEDS truncate_f64_to_u64
local function rt_divide_u64(lhs: number, rhs: number): (number, number)
	if rt_is_zero_i64(rhs) then
		error("integer divide by zero", 2)
	end

	local lhs_zeroes = rt_raw_leading_zeroes_i64(lhs)
	local rhs_zeroes = rt_raw_leading_zeroes_i64(rhs)

	if lhs_zeroes >= 11 and rhs_zeroes >= 11 then
		local lhs = rt_convert_u64_to_f64(lhs)
		local rhs = rt_convert_u64_to_f64(rhs)

		return rt_truncate_f64_to_u64(lhs / rhs), rt_truncate_f64_to_u64(lhs % rhs)
	elseif lhs_zeroes > rhs_zeroes then
		return 0, lhs
	end

	local quotient = 0
	local remainder = lhs

	for offset = rhs_zeroes - lhs_zeroes, 0, -1 do
		local offset = rt_create_i64_from_u32(offset, 0)
		local divisor = rt_shift_left_i64(rhs, offset)

		if rt_greater_than_equal_u64(remainder, divisor) == 1 then
			remainder = rt_subtract_i64(remainder, divisor)
			quotient = rt_or_i64(quotient, rt_shift_left_i64(ONE_U64, offset))
		end
	end

	return quotient, remainder
end

-- SECTION remainder_s64
-- NEEDS divide_u64
-- NEEDS is_positive
-- NEEDS negate_i64
local function rt_remainder_s64(lhs: number, rhs: number): number
	local lhs_is_positive = is_positive(lhs)
	local rhs_is_positive = is_positive(rhs)

	if not lhs_is_positive then
		lhs = rt_negate_i64(lhs)
	end

	if not rhs_is_positive then
		rhs = rt_negate_i64(rhs)
	end

	local _, remainder = rt_divide_u64(lhs, rhs)

	if not lhs_is_positive then
		remainder = rt_negate_i64(remainder)
	end

	return remainder
end

-- SECTION remainder_u64
-- NEEDS divide_u64
local function rt_remainder_u64(lhs: number, rhs: number): number
	local _, remainder = rt_divide_u64(lhs, rhs)

	return remainder
end

-- SECTION and_i64
-- NEEDS bit_and
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
local function rt_and_i64(lhs: number, rhs: number): number
	local lhs_1, lhs_2 = rt_create_u32_from_i64(lhs)
	local rhs_1, rhs_2 = rt_create_u32_from_i64(rhs)

	return rt_create_i64_from_u32(bit_and(lhs_1, rhs_1), bit_and(lhs_2, rhs_2))
end

-- SECTION or_i64
-- NEEDS bit_or
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
local function rt_or_i64(lhs: number, rhs: number): number
	local lhs_1, lhs_2 = rt_create_u32_from_i64(lhs)
	local rhs_1, rhs_2 = rt_create_u32_from_i64(rhs)

	return rt_create_i64_from_u32(bit_or(lhs_1, rhs_1), bit_or(lhs_2, rhs_2))
end

-- SECTION exclusive_or_i64
-- NEEDS bit_xor
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
local function rt_exclusive_or_i64(lhs: number, rhs: number): number
	local lhs_1, lhs_2 = rt_create_u32_from_i64(lhs)
	local rhs_1, rhs_2 = rt_create_u32_from_i64(rhs)

	return rt_create_i64_from_u32(bit_xor(lhs_1, rhs_1), bit_xor(lhs_2, rhs_2))
end

-- SECTION shift_left_i64
-- NEEDS bit_and
-- NEEDS bit_lshift
-- NEEDS bit_rshift
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
-- NEEDS create_u32_from_i64_1
local function rt_shift_left_i64(lhs: number, rhs: number): number
	local shift_1 = bit_and(rt_create_u32_from_i64_1(rhs), 0x3F)

	if shift_1 >= 32 then
		local lhs_1 = rt_create_u32_from_i64_1(lhs)
		local result_2 = bit_lshift(lhs_1, shift_1 - 32)

		return rt_create_i64_from_u32(0, result_2)
	else
		local lhs_1, lhs_2 = rt_create_u32_from_i64(lhs)
		local shift_2 = 32 - shift_1

		local result_1 = bit_lshift(lhs_1, shift_1)
		local result_2 = bit_or(bit_lshift(lhs_2, shift_1), bit_rshift(lhs_1, shift_2))

		return rt_create_i64_from_u32(result_1, result_2)
	end
end

-- SECTION shift_right_s64
-- NEEDS bit_and
-- NEEDS bit_arshift
-- NEEDS bit_lshift
-- NEEDS bit_rshift
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
-- NEEDS create_u32_from_i64_1
-- NEEDS create_u32_from_i64_2
local function rt_shift_right_s64(lhs: number, rhs: number): number
	local shift_1 = bit_and(rt_create_u32_from_i64_1(rhs), 0x3F)

	if shift_1 >= 32 then
		local lhs_2 = rt_create_u32_from_i64_2(lhs)
		local result_1 = bit_arshift(lhs_2, shift_1 - 32)
		local result_2 = if lhs_2 < 0x8000_0000 then 0 else 0xFFFF_FFFF

		return rt_create_i64_from_u32(result_1, result_2)
	else
		local lhs_1, lhs_2 = rt_create_u32_from_i64(lhs)
		local shift_2 = 32 - shift_1

		local result_1 = bit_or(bit_rshift(lhs_1, shift_1), bit_lshift(lhs_2, shift_2))
		local result_2 = bit_arshift(lhs_2, shift_1)

		return rt_create_i64_from_u32(result_1, result_2)
	end
end

-- SECTION shift_right_u64
-- NEEDS bit_and
-- NEEDS bit_lshift
-- NEEDS bit_rshift
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
-- NEEDS create_u32_from_i64_1
-- NEEDS create_u32_from_i64_2
local function rt_shift_right_u64(lhs: number, rhs: number): number
	local shift_1 = bit_and(rt_create_u32_from_i64_1(rhs), 0x3F)

	if shift_1 >= 32 then
		local lhs_2 = rt_create_u32_from_i64_2(lhs)
		local result_1 = bit_rshift(lhs_2, shift_1 - 32)

		return rt_create_i64_from_u32(result_1, 0)
	else
		local lhs_1, lhs_2 = rt_create_u32_from_i64(lhs)
		local shift_2 = 32 - shift_1

		local result_1 = bit_or(bit_rshift(lhs_1, shift_1), bit_lshift(lhs_2, shift_2))
		local result_2 = bit_rshift(lhs_2, shift_1)

		return rt_create_i64_from_u32(result_1, result_2)
	end
end

-- SECTION constant_sixty_four_i64
local SIXTY_FOUR_I64 = rt_create_i64_from_u32(64, 0)

-- SECTION rotate_left_i64
-- NEEDS constant_sixty_four_i64
-- NEEDS or_i64
-- NEEDS shift_left_i64
-- NEEDS shift_right_u64
-- NEEDS subtract_i64
local function rt_rotate_left_i64(lhs: number, rhs: number): number
	local result_1 = rt_shift_left_i64(lhs, rhs)
	local result_2 = rt_shift_right_u64(lhs, rt_subtract_i64(SIXTY_FOUR_I64, rhs))

	return rt_or_i64(result_1, result_2)
end

-- SECTION rotate_right_i64
-- NEEDS constant_sixty_four_i64
-- NEEDS or_i64
-- NEEDS shift_left_i64
-- NEEDS shift_right_u64
-- NEEDS subtract_i64
local function rt_rotate_right_i64(lhs: number, rhs: number): number
	local result_1 = rt_shift_right_u64(lhs, rhs)
	local result_2 = rt_shift_left_i64(lhs, rt_subtract_i64(SIXTY_FOUR_I64, rhs))

	return rt_or_i64(result_1, result_2)
end

-- SECTION equal_i64
-- NEEDS create_u32_from_i64
local function rt_equal_i64(lhs: number, rhs: number): number
	local lhs_1, lhs_2 = rt_create_u32_from_i64(lhs)
	local rhs_1, rhs_2 = rt_create_u32_from_i64(rhs)

	return if lhs_1 == rhs_1 and lhs_2 == rhs_2 then 1 else 0
end

-- SECTION not_equal_i64
-- NEEDS create_u32_from_i64
local function rt_not_equal_i64(lhs: number, rhs: number): number
	local lhs_1, lhs_2 = rt_create_u32_from_i64(lhs)
	local rhs_1, rhs_2 = rt_create_u32_from_i64(rhs)

	return if lhs_1 == rhs_1 and lhs_2 == rhs_2 then 0 else 1
end

-- SECTION flip_sign_bit_i64
-- NEEDS bit_xor
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
local function rt_flip_sign_bit_i64(source: number): number
	local source_1, source_2 = rt_create_u32_from_i64(source)
	local source_2 = bit_xor(source_2, 0x8000_0000)

	return rt_create_i64_from_u32(source_1, source_2)
end

-- SECTION less_than_s64
-- NEEDS flip_sign_bit_i64
-- NEEDS less_than_u64
local function rt_less_than_s64(lhs: number, rhs: number): number
	local lhs = rt_flip_sign_bit_i64(lhs)
	local rhs = rt_flip_sign_bit_i64(rhs)

	return rt_less_than_u64(lhs, rhs)
end

-- SECTION less_than_u64
-- NEEDS create_u32_from_i64
local function rt_less_than_u64(lhs: number, rhs: number): number
	local lhs_1, lhs_2 = rt_create_u32_from_i64(lhs)
	local rhs_1, rhs_2 = rt_create_u32_from_i64(rhs)

	return if lhs_2 < rhs_2 or (lhs_2 == rhs_2 and lhs_1 < rhs_1) then 1 else 0
end

-- SECTION greater_than_s64
-- NEEDS flip_sign_bit_i64
-- NEEDS greater_than_u64
local function rt_greater_than_s64(lhs: number, rhs: number): number
	local lhs = rt_flip_sign_bit_i64(lhs)
	local rhs = rt_flip_sign_bit_i64(rhs)

	return rt_greater_than_u64(lhs, rhs)
end

-- SECTION greater_than_u64
-- NEEDS create_u32_from_i64
local function rt_greater_than_u64(lhs: number, rhs: number): number
	local lhs_1, lhs_2 = rt_create_u32_from_i64(lhs)
	local rhs_1, rhs_2 = rt_create_u32_from_i64(rhs)

	return if lhs_2 > rhs_2 or (lhs_2 == rhs_2 and lhs_1 > rhs_1) then 1 else 0
end

-- SECTION less_than_equal_s64
-- NEEDS flip_sign_bit_i64
-- NEEDS less_than_equal_u64
local function rt_less_than_equal_s64(lhs: number, rhs: number): number
	local lhs = rt_flip_sign_bit_i64(lhs)
	local rhs = rt_flip_sign_bit_i64(rhs)

	return rt_less_than_equal_u64(lhs, rhs)
end

-- SECTION less_than_equal_u64
-- NEEDS equal_i64
-- NEEDS less_than_u64
local function rt_less_than_equal_u64(lhs: number, rhs: number): number
	if rt_less_than_u64(lhs, rhs) == 1 then
		return 1
	end

	return rt_equal_i64(lhs, rhs)
end

-- SECTION greater_than_equal_s64
-- NEEDS flip_sign_bit_i64
-- NEEDS greater_than_equal_u64
local function rt_greater_than_equal_s64(lhs: number, rhs: number): number
	local lhs = rt_flip_sign_bit_i64(lhs)
	local rhs = rt_flip_sign_bit_i64(rhs)

	return rt_greater_than_equal_u64(lhs, rhs)
end

-- SECTION greater_than_equal_u64
-- NEEDS equal_i64
-- NEEDS greater_than_u64
local function rt_greater_than_equal_u64(lhs: number, rhs: number): number
	if rt_greater_than_u64(lhs, rhs) == 1 then
		return 1
	end

	return rt_equal_i64(lhs, rhs)
end

-- SECTION narrow_i64
-- NEEDS create_u32_from_i64_1
local rt_narrow_i64 = rt_create_u32_from_i64_1

-- SECTION extend_s8_to_i64
-- NEEDS bit_and
-- NEEDS bit_or
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64_1
local function rt_extend_s8_to_i64(source: number): number
	local source_1 = bit_and(rt_create_u32_from_i64_1(source), 0xFF)

	return if source_1 < 0x80
		then rt_create_i64_from_u32(source_1, 0)
		else rt_create_i64_from_u32(bit_or(source_1 - 0x100, 0), 0xFFFF_FFFF)
end

-- SECTION extend_s16_to_i64
-- NEEDS bit_and
-- NEEDS bit_or
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64_1
local function rt_extend_s16_to_i64(source: number): number
	local source_1 = bit_and(rt_create_u32_from_i64_1(source), 0xFFFF)

	return if source_1 < 0x8000
		then rt_create_i64_from_u32(source_1, 0)
		else rt_create_i64_from_u32(bit_or(source_1 - 0x1_0000, 0), 0xFFFF_FFFF)
end

-- SECTION extend_s32_to_i64
-- NEEDS bit_and
-- NEEDS bit_or
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64_1
local function rt_extend_s32_to_i64(source: number): number
	local source_1 = rt_create_u32_from_i64_1(source)

	return if source_1 < 0x8000_0000
		then rt_create_i64_from_u32(source_1, 0)
		else rt_create_i64_from_u32(bit_or(source_1 - 0x1_0000_0000, 0), 0xFFFF_FFFF)
end

-- SECTION convert_s64_to_f32
-- NEEDS convert_u64_to_f32
-- NEEDS is_positive
-- NEEDS negate_i64
-- NEEDS vector_create
local function rt_convert_s64_to_f32(source: number): vector
	return if is_positive(source) then rt_convert_u64_to_f32(source) else -rt_convert_u64_to_f32(rt_negate_i64(source))
end

-- SECTION convert_u64_to_f32
-- NEEDS convert_u64_to_f64
-- NEEDS vector_create
local function rt_convert_u64_to_f32(source: number): vector
	local source = rt_convert_u64_to_f64(source)

	return vector_create(source, 0, 0)
end

-- SECTION convert_s64_to_f64
-- NEEDS convert_u64_to_f64
-- NEEDS is_positive
-- NEEDS negate_i64
local function rt_convert_s64_to_f64(source: number): vector
	return if is_positive(source) then rt_convert_u64_to_f64(source) else -rt_convert_u64_to_f64(rt_negate_i64(source))
end

-- SECTION convert_u64_to_f64
-- NEEDS create_u32_from_i64
local function rt_convert_u64_to_f64(source: number): number
	local source_1, source_2 = rt_create_u32_from_i64(source)

	return source_1 + source_2 * 0x1_0000_0000
end

-- SECTION transmute_i64_to_f64
local function rt_transmute_i64_to_f64(source: number): number
	return source
end
