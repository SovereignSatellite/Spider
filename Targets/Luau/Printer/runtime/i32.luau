-- SECTION count_ones_i32
-- NEEDS bit_and
-- NEEDS bit_rshift
local function rt_count_ones_i32(source: number): number
	local source = source - bit_and(bit_rshift(source, 1), 0x5555_5555)
	local source = bit_and(source, 0x3333_3333) + bit_and(bit_rshift(source, 2), 0x3333_3333)
	local source = bit_and(source + bit_rshift(source, 4), 0x0F0F_0F0F)
	local source = source + bit_rshift(source, 8)
	local source = source + bit_rshift(source, 16)

	return bit_and(source, 0x0000_003F)
end

-- SECTION leading_zeroes_i32
local rt_leading_zeroes_i32 = bit32.countlz

-- SECTION trailing_zeroes_i32
local rt_trailing_zeroes_i32 = bit32.countrz

-- SECTION add_i32
-- NEEDS bit_or
local function rt_add_i32(lhs: number, rhs: number): number
	return bit_or(lhs + rhs, 0)
end

-- SECTION subtract_i32
-- NEEDS bit_or
local function rt_subtract_i32(lhs: number, rhs: number): number
	return bit_or(lhs - rhs, 0)
end

-- SECTION multiply_i32
-- NEEDS bit_and
-- NEEDS bit_lshift
-- NEEDS bit_or
-- NEEDS bit_rshift
local function rt_multiply_i32(lhs: number, rhs: number): number
	if (lhs + rhs) < 0x800_0000 then
		return bit_or(lhs * rhs, 0)
	else
		local a16 = bit_rshift(lhs, 16)
		local a00 = bit_and(lhs, 0xFFFF)
		local b16 = bit_rshift(rhs, 16)
		local b00 = bit_and(rhs, 0xFFFF)

		local c00 = a00 * b00
		local c16 = a16 * b00 + a00 * b16

		return bit_or(c00 + bit_lshift(c16, 16), 0)
	end
end

-- SECTION divide_s32
-- NEEDS bit_or
-- NEEDS bit_xor
local function rt_divide_s32(lhs: number, rhs: number): number
	if rhs == 0 then
		error("integer divide by zero", 2)
	elseif lhs == 0x80000000 and rhs == 0xFFFFFFFF then
		error("integer overflow", 2)
	end

	local lhs = bit_xor(lhs, 0x8000_0000) - 0x8000_0000
	local rhs = bit_xor(rhs, 0x8000_0000) - 0x8000_0000

	return bit_or(lhs / rhs, 0)
end

-- SECTION divide_u32
-- NEEDS bit_or
local function rt_divide_u32(lhs: number, rhs: number): number
	if rhs == 0 then
		error("integer divide by zero", 2)
	end

	return bit_or(lhs / rhs, 0)
end

-- SECTION remainder_s32
-- NEEDS bit_or
-- NEEDS bit_xor
-- NEEDS math_fmod
local function rt_remainder_s32(lhs: number, rhs: number): number
	if rhs == 0 then
		error("integer divide by zero", 2)
	end

	local lhs = bit_xor(lhs, 0x8000_0000) - 0x8000_0000
	local rhs = bit_xor(rhs, 0x8000_0000) - 0x8000_0000

	return bit_or(math_fmod(lhs, rhs), 0)
end

-- SECTION remainder_u32
-- NEEDS bit_or
local function rt_remainder_u32(lhs: number, rhs: number): number
	if rhs == 0 then
		error("integer divide by zero", 2)
	end

	return bit_or(lhs % rhs, 0)
end

-- SECTION and_i32
local rt_and_i32 = bit32.band

-- SECTION or_i32
local rt_or_i32 = bit32.bor

-- SECTION exclusive_or_i32
local rt_exclusive_or_i32 = bit32.bxor

-- SECTION shift_left_i32
-- NEEDS bit_and
-- NEEDS bit_lshift
local function rt_shift_left_i32(lhs: number, rhs: number): number
	return bit_lshift(lhs, bit_and(rhs, 0x1F))
end

-- SECTION shift_right_s32
-- NEEDS bit_and
-- NEEDS bit_arshift
local function rt_shift_right_s32(lhs: number, rhs: number): number
	return bit_arshift(lhs, bit_and(rhs, 0x1F))
end

-- SECTION shift_right_u32
-- NEEDS bit_and
-- NEEDS bit_rshift
local function rt_shift_right_u32(lhs: number, rhs: number): number
	return bit_rshift(lhs, bit_and(rhs, 0x1F))
end

-- SECTION rotate_left_i32
-- NEEDS bit_and
-- NEEDS bit_lrotate
local function rt_rotate_left_i32(lhs: number, rhs: number): number
	return bit_lrotate(lhs, bit_and(rhs, 0x1F))
end

-- SECTION rotate_right_i32
-- NEEDS bit_and
-- NEEDS bit_rrotate
local function rt_rotate_right_i32(lhs: number, rhs: number): number
	return bit_rrotate(lhs, bit_and(rhs, 0x1F))
end

-- SECTION equal_i32
local function rt_equal_i32(lhs: number, rhs: number): number
	return if lhs == rhs then 1 else 0
end

-- SECTION not_equal_i32
local function rt_not_equal_i32(lhs: number, rhs: number): number
	return if lhs == rhs then 0 else 1
end

-- SECTION less_than_s32
-- NEEDS bit_xor
local function rt_less_than_s32(lhs: number, rhs: number): number
	return if bit_xor(lhs, 0x8000_0000) < bit_xor(rhs, 0x8000_0000) then 1 else 0
end

-- SECTION less_than_u32
local function rt_less_than_u32(lhs: number, rhs: number): number
	return if lhs < rhs then 1 else 0
end

-- SECTION greater_than_s32
-- NEEDS bit_xor
local function rt_greater_than_s32(lhs: number, rhs: number): number
	return if bit_xor(lhs, 0x8000_0000) > bit_xor(rhs, 0x8000_0000) then 1 else 0
end

-- SECTION greater_than_u32
local function rt_greater_than_u32(lhs: number, rhs: number): number
	return if lhs > rhs then 1 else 0
end

-- SECTION less_than_equal_s32
-- NEEDS bit_xor
local function rt_less_than_equal_s32(lhs: number, rhs: number): number
	return if bit_xor(lhs, 0x8000_0000) <= bit_xor(rhs, 0x8000_0000) then 1 else 0
end

-- SECTION less_than_equal_u32
local function rt_less_than_equal_u32(lhs: number, rhs: number): number
	return if lhs <= rhs then 1 else 0
end

-- SECTION greater_than_equal_s32
-- NEEDS bit_xor
local function rt_greater_than_equal_s32(lhs: number, rhs: number): number
	return if bit_xor(lhs, 0x8000_0000) >= bit_xor(rhs, 0x8000_0000) then 1 else 0
end

-- SECTION greater_than_equal_u32
local function rt_greater_than_equal_u32(lhs: number, rhs: number): number
	return if lhs >= rhs then 1 else 0
end

-- SECTION widen_i32
-- NEEDS create_i64_from_u32
local function rt_widen_i32(source: number): number
	return rt_create_i64_from_u32(source, 0)
end

-- SECTION extend_s8_to_i32
-- NEEDS bit_and
-- NEEDS bit_or
local function rt_extend_s8_to_i32(source: number): number
	local source = bit_and(source, 0xFF)

	return if source < 0x80 then source else bit_or(source - 0x100, 0)
end

-- SECTION extend_s16_to_i32
-- NEEDS bit_and
-- NEEDS bit_or
local function rt_extend_s16_to_i32(source: number): number
	local source = bit_and(source, 0xFFFF)

	return if source < 0x8000 then source else bit_or(source - 0x1_0000, 0)
end

-- SECTION convert_s32_to_f32
-- NEEDS bit_xor
-- NEEDS vector_create
local function rt_convert_s32_to_f32(source: number): vector
	local source = bit_xor(source, 0x8000_0000) - 0x8000_0000

	return vector_create(source, 0, 0)
end

-- SECTION convert_u32_to_f32
-- NEEDS vector_create
local function rt_convert_u32_to_f32(source: number): vector
	return vector_create(source, 0, 0)
end

-- SECTION convert_s32_to_f64
-- NEEDS bit_xor
local function rt_convert_s32_to_f64(source: number): number
	local source = bit_xor(source, 0x8000_0000) - 0x8000_0000

	return source
end

-- SECTION convert_u32_to_f64
local function rt_convert_u32_to_f64(source: number): number
	return source
end

-- SECTION transmute_i32_to_f32
-- NEEDS buffer_read_f32
-- NEEDS buffer_write_u32
-- NEEDS raw_memory
-- NEEDS vector_create
local function rt_transmute_i32_to_f32(source: number): vector
	buffer_write_u32(RAW_MEMORY, 0, source)

	return vector_create(buffer_read_f32(RAW_MEMORY, 0), 0, 0)
end
