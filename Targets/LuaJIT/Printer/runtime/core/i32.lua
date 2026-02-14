-- SECTION count_ones_i32
-- NEEDS bit_and
-- NEEDS bit_rshift
local function rt_count_ones_i32(source)
	source = source - bit_and(bit_rshift(source, 1), 0x55555555)
	source = bit_and(source, 0x33333333) + bit_and(bit_rshift(source, 2), 0x33333333)
	source = bit_and(source + bit_rshift(source, 4), 0x0F0F0F0F)
	source = source + bit_rshift(source, 8)
	source = source + bit_rshift(source, 16)
	source = bit_and(source, 0x0000003F)

	return source
end

-- SECTION leading_zeroes_i32
-- NEEDS bit_lshift
-- NEEDS bit_rshift
local function rt_leading_zeroes_i32(source)
	if source == 0 then
		return 32
	end

	local result = 0

	if bit_rshift(source, 16) == 0 then
		source = bit_lshift(source, 16)
		result = result + 16
	end

	if bit_rshift(source, 24) == 0 then
		source = bit_lshift(source, 8)
		result = result + 8
	end

	if bit_rshift(source, 28) == 0 then
		source = bit_lshift(source, 4)
		result = result + 4
	end

	if bit_rshift(source, 30) == 0 then
		source = bit_lshift(source, 2)
		result = result + 2
	end

	if bit_rshift(source, 31) == 0 then
		result = result + 1
	end

	return result
end

-- SECTION trailing_zeroes_i32
-- NEEDS bit_lshift
-- NEEDS bit_rshift
local function rt_trailing_zeroes_i32(source)
	if source == 0 then
		return 32
	end

	local result = 0

	if bit_lshift(source, 16) == 0 then
		source = bit_rshift(source, 16)
		result = result + 16
	end

	if bit_lshift(source, 24) == 0 then
		source = bit_rshift(source, 8)
		result = result + 8
	end

	if bit_lshift(source, 28) == 0 then
		source = bit_rshift(source, 4)
		result = result + 4
	end

	if bit_lshift(source, 30) == 0 then
		source = bit_rshift(source, 2)
		result = result + 2
	end

	if bit_lshift(source, 31) == 0 then
		result = result + 1
	end

	return result
end

-- SECTION add_i32
-- NEEDS force_i32
local function rt_add_i32(lhs, rhs)
	local result = lhs + rhs

	result = force_i32(result)

	return result
end

-- SECTION subtract_i32
-- NEEDS force_i32
local function rt_subtract_i32(lhs, rhs)
	local result = lhs - rhs

	result = force_i32(result)

	return result
end

-- SECTION multiply_i32
-- NEEDS ffi_cast
-- NEEDS force_i32
-- NEEDS i64_type
local function rt_multiply_i32(lhs, rhs)
	lhs = ffi_cast(i64_type, lhs)
	rhs = ffi_cast(i64_type, rhs)

	local result = lhs * rhs

	result = force_i32(result)

	return result
end

-- SECTION divide_s32
-- NEEDS force_i32
-- NEEDS math_modf
local function rt_divide_s32(lhs, rhs)
	if lhs == -0x80000000 and rhs == -1 then
		error("integer overflow")
	elseif rhs == 0 then
		error("integer divide by zero")
	end

	local result = lhs / rhs

	result = math_modf(result)
	result = force_i32(result)

	return result
end

-- SECTION divide_u32
-- NEEDS force_i32
-- NEEDS force_u32
-- NEEDS math_floor
local function rt_divide_u32(lhs, rhs)
	if rhs == 0 then
		error("integer divide by zero")
	end

	lhs = force_u32(lhs)
	rhs = force_u32(rhs)

	local result = lhs / rhs

	result = math_floor(result)
	result = force_i32(result)

	return result
end

-- SECTION remainder_s32
-- NEEDS force_i32
-- NEEDS math_fmod
local function rt_remainder_s32(lhs, rhs)
	if rhs == 0 then
		error("integer divide by zero")
	end

	local result = math_fmod(lhs, rhs)

	result = force_i32(result)

	return result
end

-- SECTION remainder_u32
-- NEEDS force_i32
-- NEEDS force_u32
local function rt_remainder_u32(lhs, rhs)
	if rhs == 0 then
		error("integer divide by zero", 2)
	end

	lhs = force_u32(lhs)
	rhs = force_u32(rhs)

	local result = lhs % rhs

	result = force_i32(result)

	return result
end

-- SECTION and_i32
-- NEEDS bit
local rt_and_i32 = bit.band

-- SECTION or_i32
-- NEEDS bit
local rt_or_i32 = bit.bor

-- SECTION exclusive_or_i32
-- NEEDS bit
local rt_exclusive_or_i32 = bit.bxor

-- SECTION shift_left_i32
-- NEEDS bit_and
-- NEEDS bit_lshift
local function rt_shift_left_i32(lhs, rhs)
	rhs = bit_and(rhs, 0x1F)

	local result = bit_lshift(lhs, rhs)

	return result
end

-- SECTION shift_right_s32
-- NEEDS bit_and
-- NEEDS bit_arshift
local function rt_shift_right_s32(lhs, rhs)
	rhs = bit_and(rhs, 0x1F)

	local result = bit_arshift(lhs, rhs)

	return result
end

-- SECTION shift_right_u32
-- NEEDS bit_and
-- NEEDS bit_rshift
local function rt_shift_right_u32(lhs, rhs)
	rhs = bit_and(rhs, 0x1F)

	local result = bit_rshift(lhs, rhs)

	return result
end

-- SECTION rotate_left_i32
-- NEEDS bit_and
-- NEEDS bit_lrotate
local function rt_rotate_left_i32(lhs, rhs)
	rhs = bit_and(rhs, 0x1F)

	local result = bit_lrotate(lhs, rhs)

	return result
end

-- SECTION rotate_right_i32
-- NEEDS bit_and
-- NEEDS bit_rrotate
local function rt_rotate_right_i32(lhs, rhs)
	rhs = bit_and(rhs, 0x1F)

	local result = bit_rrotate(lhs, rhs)

	return result
end

-- SECTION equal_i32
local function rt_equal_i32(lhs, rhs)
	return lhs == rhs
end

-- SECTION not_equal_i32
local function rt_not_equal_i32(lhs, rhs)
	return lhs ~= rhs
end

-- SECTION less_than_s32
local function rt_less_than_s32(lhs, rhs)
	return lhs < rhs
end

-- SECTION less_than_u32
-- NEEDS bit_xor
local function rt_less_than_u32(lhs, rhs)
	lhs = bit_xor(lhs, 0x80000000)
	rhs = bit_xor(rhs, 0x80000000)

	return lhs < rhs
end

-- SECTION greater_than_s32
local function rt_greater_than_s32(lhs, rhs)
	return lhs > rhs
end

-- SECTION greater_than_u32
-- NEEDS bit_xor
local function rt_greater_than_u32(lhs, rhs)
	lhs = bit_xor(lhs, 0x80000000)
	rhs = bit_xor(rhs, 0x80000000)

	return lhs > rhs
end

-- SECTION less_than_equal_s32
local function rt_less_than_equal_s32(lhs, rhs)
	return lhs <= rhs
end

-- SECTION less_than_equal_u32
-- NEEDS bit_xor
local function rt_less_than_equal_u32(lhs, rhs)
	lhs = bit_xor(lhs, 0x80000000)
	rhs = bit_xor(rhs, 0x80000000)

	return lhs <= rhs
end

-- SECTION greater_than_equal_s32
local function rt_greater_than_equal_s32(lhs, rhs)
	return lhs >= rhs
end

-- SECTION greater_than_equal_u32
-- NEEDS bit_xor
local function rt_greater_than_equal_u32(lhs, rhs)
	lhs = bit_xor(lhs, 0x80000000)
	rhs = bit_xor(rhs, 0x80000000)

	return lhs >= rhs
end

-- SECTION widen_i32
-- NEEDS ffi_cast
-- NEEDS i64_type
local function rt_widen_i32(source)
	source = ffi_cast(i64_type, source)

	if source < 0LL then
		source = source + 0x100000000LL
	end

	return source
end

-- SECTION extend_s8_to_i32
-- NEEDS bit_and
-- NEEDS force_i32
local function rt_extend_s8_to_i32(source)
	source = bit_and(source, 0xFF)

	if source >= 0x80 then
		source = force_i32(source - 0x100)
	end

	return source
end

-- SECTION extend_s16_to_i32
-- NEEDS bit_and
-- NEEDS force_i32
local function rt_extend_s16_to_i32(source)
	source = bit_and(source, 0xFFFF)

	if source >= 0x8000 then
		source = force_i32(source - 0x10000)
	end

	return source
end

-- SECTION convert_s32_to_f32
-- NEEDS transmute_n32
local function rt_convert_s32_to_f32(source)
	TRANSMUTE_N32.f32 = source

	return TRANSMUTE_N32.i32
end

-- SECTION convert_u32_to_f32
-- NEEDS force_u32
-- NEEDS transmute_n32
local function rt_convert_u32_to_f32(source)
	source = force_u32(source)

	TRANSMUTE_N32.f32 = source

	return TRANSMUTE_N32.i32
end

-- SECTION convert_s32_to_f64
-- NEEDS transmute_n64
local function rt_convert_s32_to_f64(source)
	TRANSMUTE_N64.f64 = source

	return TRANSMUTE_N64.i64
end

-- SECTION convert_u32_to_f64
-- NEEDS force_u32
-- NEEDS transmute_n64
local function rt_convert_u32_to_f64(source)
	source = force_u32(source)

	TRANSMUTE_N64.f64 = source

	return TRANSMUTE_N64.i64
end

-- SECTION transmute_i32_to_f32
local function rt_transmute_i32_to_f32(source)
	return source
end
