-- SECTION absolute_f32
-- NEEDS bit32_and
local function rt_absolute_f32(source: number): number
	source = bit32_and(source, 0x7FFF_FFFF)

	return source
end

-- SECTION negate_f32
-- NEEDS bit32_xor
local function rt_negate_f32(source: number): number
	source = bit32_xor(source, 0x8000_0000)

	return source
end

-- SECTION square_root_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_sqrt
local function rt_square_root_f32(source: number): number
	source = from_bits_f32(source)
	source = math_sqrt(source)
	source = into_bits_f32(source)

	return source
end

-- SECTION round_up_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_ceil
local function rt_round_up_f32(source: number): number
	source = from_bits_f32(source)
	source = math_ceil(source)
	source = into_bits_f32(source)

	return source
end

-- SECTION round_down_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_floor
local function rt_round_down_f32(source: number): number
	source = from_bits_f32(source)
	source = math_floor(source)
	source = into_bits_f32(source)

	return source
end

-- SECTION truncate_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_modf
local function rt_truncate_f32(source: number): number
	source = from_bits_f32(source)
	source = math_modf(source)
	source = into_bits_f32(source)

	return source
end

-- SECTION nearest_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_abs
-- NEEDS math_modf
local function rt_nearest_f32(source: number): number
	local native = from_bits_f32(source)

	native = math_abs(native)

	local rounded, remainder = math_modf(native)

	if remainder > 0.5 or (remainder == 0.5 and rounded % 2 == 1) then
		rounded = rounded + 1
	end

	if source >= 0x8000_0000 then
		rounded = -rounded
	end

	source = into_bits_f32(rounded)

	return source
end

-- SECTION add_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS vector_create
local function rt_add_f32(lhs: number, rhs: number): number
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	local result = vector_create(lhs, 0, 0) + vector_create(rhs, 0, 0)

	result = into_bits_f32(result.x)

	return result
end

-- SECTION subtract_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS vector_create
local function rt_subtract_f32(lhs: number, rhs: number): number
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	local result = vector_create(lhs, 0, 0) - vector_create(rhs, 0, 0)

	result = into_bits_f32(result.x)

	return result
end

-- SECTION multiply_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS vector_create
local function rt_multiply_f32(lhs: number, rhs: number): number
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	local result = vector_create(lhs, 0, 0) * vector_create(rhs, 0, 0)

	result = into_bits_f32(result.x)

	return result
end

-- SECTION divide_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS vector_create
local function rt_divide_f32(lhs: number, rhs: number): number
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	local result = vector_create(lhs, 0, 0) / vector_create(rhs, 0, 0)

	result = into_bits_f32(result.x)

	return result
end

-- SECTION minimum_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_min
local function rt_minimum_f32(lhs: number, rhs: number): number
	if lhs < 0x8000_0000 then
		lhs, rhs = rhs, lhs
	end

	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	local result = math_min(lhs, rhs)

	result = into_bits_f32(result)

	return result
end

-- SECTION maximum_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_max
local function rt_maximum_f32(lhs: number, rhs: number): number
	if rhs < 0x8000_0000 then
		lhs, rhs = rhs, lhs
	end

	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	local result = math_max(lhs, rhs)

	result = into_bits_f32(result)

	return result
end

-- SECTION copy_sign_f32
-- NEEDS bit32_and
-- NEEDS bit32_or
local function rt_copy_sign_f32(lhs: number, rhs: number): number
	lhs = bit32_and(lhs, 0x7FFF_FFFF)
	rhs = bit32_and(rhs, 0x8000_0000)

	local result = bit32_or(lhs, rhs)

	return result
end

-- SECTION equal_f32
-- NEEDS from_bits_f32
local function rt_equal_f32(lhs: number, rhs: number): boolean
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs == rhs
end

-- SECTION not_equal_f32
-- NEEDS from_bits_f32
local function rt_not_equal_f32(lhs: number, rhs: number): boolean
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs ~= rhs
end

-- SECTION less_than_f32
-- NEEDS from_bits_f32
local function rt_less_than_f32(lhs: number, rhs: number): boolean
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs < rhs
end

-- SECTION greater_than_f32
-- NEEDS from_bits_f32
local function rt_greater_than_f32(lhs: number, rhs: number): boolean
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs > rhs
end

-- SECTION less_than_equal_f32
-- NEEDS from_bits_f32
local function rt_less_than_equal_f32(lhs: number, rhs: number): boolean
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs <= rhs
end

-- SECTION greater_than_equal_f32
-- NEEDS from_bits_f32
local function rt_greater_than_equal_f32(lhs: number, rhs: number): boolean
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs >= rhs
end

-- SECTION widen_f32
-- NEEDS buffer_read_f32
-- NEEDS buffer_write_u32
-- NEEDS transmute_n64
local function rt_widen_f32(source: number): number
	buffer_write_u32(TRANSMUTE_N64, 0, source)

	return buffer_read_f32(TRANSMUTE_N64, 0)
end

-- SECTION saturate_f32_to_s32
-- NEEDS bit32_or
-- NEEDS from_bits_f32
-- NEEDS math_modf
local function rt_saturate_f32_to_s32(source: number): number
	source = from_bits_f32(source)

	if source >= 0x8000_0000 then
		source = 0x7FFF_FFFF
	elseif source <= -0x8000_0000 then
		source = 0x8000_0000
	elseif source ~= source then
		source = 0
	else
		source = math_modf(source)
		source = bit32_or(source, 0)
	end

	return source
end

-- SECTION truncate_f32_to_s32
-- NEEDS bit32_or
-- NEEDS from_bits_f32
-- NEEDS math_modf
local function rt_truncate_f32_to_s32(source: number): number
	source = from_bits_f32(source)
	source = math_modf(source)

	if source >= 0x8000_0000 or source < -0x8000_0000 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = bit32_or(source, 0)

	return source
end

-- SECTION saturate_f32_to_u32
-- NEEDS bit32_or
-- NEEDS from_bits_f32
-- NEEDS math_floor
local function rt_saturate_f32_to_u32(source: number): number
	source = from_bits_f32(source)

	if source >= 0x1_0000_0000 then
		source = 0xFFFF_FFFF
	elseif source <= 0 or source ~= source then
		source = 0
	else
		source = math_floor(source)
	end

	source = bit32_or(source, 0)

	return source
end

-- SECTION truncate_f32_to_u32
-- NEEDS bit32_or
-- NEEDS from_bits_f32
-- NEEDS math_modf
local function rt_truncate_f32_to_u32(source: number): number
	source = from_bits_f32(source)
	source = math_modf(source)

	if source >= 0x1_0000_0000 or source < 0 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = bit32_or(source, 0)

	return source
end

-- SECTION saturate_f32_to_s64
-- NEEDS from_bits_f32
-- NEEDS integer_create
-- NEEDS math_modf
local function rt_saturate_f32_to_s64(source: number): integer
	source = from_bits_f32(source)

	if source >= 0x8000_0000_0000_0000 then
		source = 0x7FFF_FFFF_FFFF_FFFFi
	elseif source <= -0x8000_0000_0000_0000 then
		source = 0x8000_0000_0000_0000i
	elseif source ~= source then
		source = 0x0000_0000_0000_0000i
	else
		source = math_modf(source)
		source = integer_create(source)
	end

	return source
end

-- SECTION truncate_f32_to_s64
-- NEEDS from_bits_f32
-- NEEDS integer_create
-- NEEDS math_modf
local function rt_truncate_f32_to_s64(source: number): integer
	source = from_bits_f32(source)
	source = math_modf(source)

	if source >= 0x8000_0000_0000_0000 or source < -0x8000_0000_0000_0000 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = integer_create(source)

	return source
end

-- SECTION saturate_f32_to_u64
-- NEEDS convert_f64_to_u64
-- NEEDS from_bits_f32
-- NEEDS math_floor
local function rt_saturate_f32_to_u64(source: number): integer
	source = from_bits_f32(source)

	if source >= 0x1_0000_0000_0000_0000 then
		source = 0xFFFF_FFFF_FFFF_FFFFi
	elseif source <= 0 or source ~= source then
		source = 0x0000_0000_0000_0000i
	else
		source = math_floor(source)
		source = convert_f64_to_u64(source)
	end

	return source
end

-- SECTION truncate_f32_to_u64
-- NEEDS convert_f64_to_u64
-- NEEDS from_bits_f32
-- NEEDS math_modf
local function rt_truncate_f32_to_u64(source: number): integer
	source = from_bits_f32(source)
	source = math_modf(source)

	if source >= 0x1_0000_0000_0000_0000 or source < 0 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = convert_f64_to_u64(source)

	return source
end

-- SECTION transmute_f32_to_i32
local function rt_transmute_f32_to_i32(source: number): number
	return source
end
