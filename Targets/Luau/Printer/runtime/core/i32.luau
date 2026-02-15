-- SECTION count_ones_i32
-- NEEDS bit32_and
-- NEEDS bit32_rshift
local function rt_count_ones_i32(source: number): number
	source = source - bit32_and(bit32_rshift(source, 1), 0x5555_5555)
	source = bit32_and(source, 0x3333_3333) + bit32_and(bit32_rshift(source, 2), 0x3333_3333)
	source = bit32_and(source + bit32_rshift(source, 4), 0x0F0F_0F0F)
	source = source + bit32_rshift(source, 8)
	source = source + bit32_rshift(source, 16)
	source = bit32_and(source, 0x0000_003F)

	return source
end

-- SECTION leading_zeroes_i32
local rt_leading_zeroes_i32 = bit32.countlz

-- SECTION trailing_zeroes_i32
local rt_trailing_zeroes_i32 = bit32.countrz

-- SECTION add_i32
-- NEEDS bit32_or
local function rt_add_i32(lhs: number, rhs: number): number
	local result = bit32_or(lhs + rhs, 0)

	return result
end

-- SECTION subtract_i32
-- NEEDS bit32_or
local function rt_subtract_i32(lhs: number, rhs: number): number
	local result = bit32_or(lhs - rhs, 0)

	return result
end

-- SECTION multiply_i32
-- NEEDS integer_and
-- NEEDS integer_create
-- NEEDS integer_mul
-- NEEDS integer_tonumber
local function rt_multiply_i32(lhs: number, rhs: number): number
	lhs = integer_create(lhs)
	rhs = integer_create(rhs)

	local result = integer_mul(lhs, rhs)

	result = integer_and(result, 0xFFFF_FFFFi)
	result = integer_tonumber(result)

	return result
end

-- SECTION divide_s32
-- NEEDS bit32_or
-- NEEDS bit32_xor
-- NEEDS math_modf
local function rt_divide_s32(lhs: number, rhs: number): number
	if lhs == 0x8000_0000 and rhs == 0xFFFF_FFFF then
		error("integer overflow")
	elseif rhs == 0 then
		error("integer divide by zero")
	end

	lhs = bit32_xor(lhs, 0x8000_0000) - 0x8000_0000
	rhs = bit32_xor(rhs, 0x8000_0000) - 0x8000_0000

	local result = lhs / rhs

	result = math_modf(result)
	result = bit32_or(result, 0)

	return result
end

-- SECTION divide_u32
local function rt_divide_u32(lhs: number, rhs: number): number
	if rhs == 0 then
		error("integer divide by zero")
	end

	return lhs // rhs
end

-- SECTION remainder_s32
-- NEEDS bit32_or
-- NEEDS bit32_xor
-- NEEDS math_fmod
local function rt_remainder_s32(lhs: number, rhs: number): number
	if rhs == 0 then
		error("integer divide by zero", 2)
	end

	lhs = bit32_xor(lhs, 0x8000_0000) - 0x8000_0000
	rhs = bit32_xor(rhs, 0x8000_0000) - 0x8000_0000

	local result = math_fmod(lhs, rhs)

	result = bit32_or(result, 0)

	return result
end

-- SECTION remainder_u32
-- NEEDS bit32_or
local function rt_remainder_u32(lhs: number, rhs: number): number
	if rhs == 0 then
		error("integer divide by zero", 2)
	end

	local result = lhs % rhs

	result = bit32_or(result, 0)

	return result
end

-- SECTION and_i32
local rt_and_i32 = bit32.band

-- SECTION or_i32
local rt_or_i32 = bit32.bor

-- SECTION exclusive_or_i32
local rt_exclusive_or_i32 = bit32.bxor

-- SECTION shift_left_i32
-- NEEDS bit32_and
-- NEEDS bit32_lshift
local function rt_shift_left_i32(lhs: number, rhs: number): number
	rhs = bit32_and(rhs, 0x1F)

	local result = bit32_lshift(lhs, rhs)

	return result
end

-- SECTION shift_right_s32
-- NEEDS bit32_and
-- NEEDS bit32_arshift
local function rt_shift_right_s32(lhs: number, rhs: number): number
	rhs = bit32_and(rhs, 0x1F)

	local result = bit32_arshift(lhs, rhs)

	return result
end

-- SECTION shift_right_u32
-- NEEDS bit32_and
-- NEEDS bit32_rshift
local function rt_shift_right_u32(lhs: number, rhs: number): number
	rhs = bit32_and(rhs, 0x1F)

	local result = bit32_rshift(lhs, rhs)

	return result
end

-- SECTION rotate_left_i32
-- NEEDS bit32_and
-- NEEDS bit32_lrotate
local function rt_rotate_left_i32(lhs: number, rhs: number): number
	rhs = bit32_and(rhs, 0x1F)

	local result = bit32_lrotate(lhs, rhs)

	return result
end

-- SECTION rotate_right_i32
-- NEEDS bit32_and
-- NEEDS bit32_rrotate
local function rt_rotate_right_i32(lhs: number, rhs: number): number
	rhs = bit32_and(rhs, 0x1F)

	local result = bit32_rrotate(lhs, rhs)

	return result
end

-- SECTION equal_i32
local function rt_equal_i32(lhs: number, rhs: number): boolean
	return lhs == rhs
end

-- SECTION not_equal_i32
local function rt_not_equal_i32(lhs: number, rhs: number): boolean
	return lhs ~= rhs
end

-- SECTION less_than_s32
-- NEEDS bit32_xor
local function rt_less_than_s32(lhs: number, rhs: number): boolean
	lhs = bit32_xor(lhs, 0x8000_0000)
	rhs = bit32_xor(rhs, 0x8000_0000)

	return lhs < rhs
end

-- SECTION less_than_u32
local function rt_less_than_u32(lhs: number, rhs: number): boolean
	return lhs < rhs
end

-- SECTION greater_than_s32
-- NEEDS bit32_xor
local function rt_greater_than_s32(lhs: number, rhs: number): boolean
	lhs = bit32_xor(lhs, 0x8000_0000)
	rhs = bit32_xor(rhs, 0x8000_0000)

	return lhs > rhs
end

-- SECTION greater_than_u32
local function rt_greater_than_u32(lhs: number, rhs: number): boolean
	return lhs > rhs
end

-- SECTION less_than_equal_s32
-- NEEDS bit32_xor
local function rt_less_than_equal_s32(lhs: number, rhs: number): boolean
	lhs = bit32_xor(lhs, 0x8000_0000)
	rhs = bit32_xor(rhs, 0x8000_0000)

	return lhs <= rhs
end

-- SECTION less_than_equal_u32
local function rt_less_than_equal_u32(lhs: number, rhs: number): boolean
	return lhs <= rhs
end

-- SECTION greater_than_equal_s32
-- NEEDS bit32_xor
local function rt_greater_than_equal_s32(lhs: number, rhs: number): boolean
	lhs = bit32_xor(lhs, 0x8000_0000)
	rhs = bit32_xor(rhs, 0x8000_0000)

	return lhs >= rhs
end

-- SECTION greater_than_equal_u32
local function rt_greater_than_equal_u32(lhs: number, rhs: number): boolean
	return lhs >= rhs
end

-- SECTION widen_i32
local rt_widen_i32 = integer.create

-- SECTION extend_s8_to_i32
-- NEEDS bit32_and
-- NEEDS bit32_or
local function rt_extend_s8_to_i32(source: number): number
	source = bit32_and(source, 0xFF)

	if source >= 0x80 then
		source = bit32_or(source - 0x100, 0)
	end

	return source
end

-- SECTION extend_s16_to_i32
-- NEEDS bit32_and
-- NEEDS bit32_or
local function rt_extend_s16_to_i32(source: number): number
	source = bit32_and(source, 0xFFFF)

	if source >= 0x8000 then
		source = bit32_or(source - 0x1_0000, 0)
	end

	return source
end

-- SECTION convert_s32_to_f32
-- NEEDS bit32_xor
-- NEEDS buffer_read_u32
-- NEEDS buffer_write_f32
-- NEEDS transmute_n64
local function rt_convert_s32_to_f32(source: number): number
	source = bit32_xor(source, 0x8000_0000) - 0x8000_0000

	buffer_write_f32(TRANSMUTE_N64, 0, source)

	source = buffer_read_u32(TRANSMUTE_N64, 0)

	return source
end

-- SECTION convert_u32_to_f32
-- NEEDS buffer_read_u32
-- NEEDS buffer_write_f32
-- NEEDS transmute_n64
local function rt_convert_u32_to_f32(source: number): number
	buffer_write_f32(TRANSMUTE_N64, 0, source)

	source = buffer_read_u32(TRANSMUTE_N64, 0)

	return source
end

-- SECTION convert_s32_to_f64
-- NEEDS bit32_xor
local function rt_convert_s32_to_f64(source: number): number
	source = bit32_xor(source, 0x8000_0000) - 0x8000_0000

	return source
end

-- SECTION convert_u32_to_f64
local function rt_convert_u32_to_f64(source: number): number
	return source
end

-- SECTION transmute_i32_to_f32
local function rt_transmute_i32_to_f32(source: number): number
	return source
end
