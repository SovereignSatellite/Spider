-- SECTION absolute_f32
-- NEEDS bit_and
local function rt_absolute_f32(source)
	source = bit_and(source, 0x7FFFFFFF)

	return source
end

-- SECTION negate_f32
-- NEEDS bit_xor
local function rt_negate_f32(source)
	source = bit_xor(source, 0x80000000)

	return source
end

-- SECTION square_root_f32
-- NEEDS native_f32
local rt_square_root_f32 = NATIVE_F32.square_root_f32

-- SECTION round_up_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_ceil
local function rt_round_up_f32(source)
	source = from_bits_f32(source)
	source = math_ceil(source)
	source = into_bits_f32(source)

	return source
end

-- SECTION round_down_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_floor
local function rt_round_down_f32(source)
	source = from_bits_f32(source)
	source = math_floor(source)
	source = into_bits_f32(source)

	return source
end

-- SECTION truncate_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_modf
local function rt_truncate_f32(source)
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
local function rt_nearest_f32(source)
	local native = from_bits_f32(source)

	native = math_abs(native)

	local rounded, remainder = math_modf(native)

	if remainder > 0.5 or (remainder == 0.5 and rounded % 2 == 1) then
		rounded = rounded + 1
	end

	if source < 0 then
		rounded = -rounded
	end

	source = into_bits_f32(rounded)

	return source
end

-- SECTION add_f32
-- NEEDS native_f32
local rt_add_f32 = NATIVE_F32.add_f32

-- SECTION subtract_f32
-- NEEDS native_f32
local rt_subtract_f32 = NATIVE_F32.subtract_f32

-- SECTION multiply_f32
-- NEEDS native_f32
local rt_multiply_f32 = NATIVE_F32.multiply_f32

-- SECTION divide_f32
-- NEEDS native_f32
local rt_divide_f32 = NATIVE_F32.divide_f32

-- SECTION minimum_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_min
local function rt_minimum_f32(lhs, rhs)
	if rhs >= 0 then
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
local function rt_maximum_f32(lhs, rhs)
	if rhs < 0 then
		lhs, rhs = rhs, lhs
	end

	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	local result = math_max(lhs, rhs)

	result = into_bits_f32(result)

	return result
end

-- SECTION copy_sign_f32
-- NEEDS bit_and
-- NEEDS bit_or
local function rt_copy_sign_f32(lhs, rhs)
	lhs = bit_and(lhs, 0x7FFFFFFF)
	rhs = bit_and(rhs, 0x80000000)

	local result = bit_or(lhs, rhs)

	return result
end

-- SECTION equal_f32
-- NEEDS from_bits_f32
local function rt_equal_f32(lhs, rhs)
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs == rhs
end

-- SECTION not_equal_f32
-- NEEDS from_bits_f32
local function rt_not_equal_f32(lhs, rhs)
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs ~= rhs
end

-- SECTION less_than_f32
-- NEEDS from_bits_f32
local function rt_less_than_f32(lhs, rhs)
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs < rhs
end

-- SECTION greater_than_f32
-- NEEDS from_bits_f32
local function rt_greater_than_f32(lhs, rhs)
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs > rhs
end

-- SECTION less_than_equal_f32
-- NEEDS from_bits_f32
local function rt_less_than_equal_f32(lhs, rhs)
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs <= rhs
end

-- SECTION greater_than_equal_f32
-- NEEDS from_bits_f32
local function rt_greater_than_equal_f32(lhs, rhs)
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs >= rhs
end

-- SECTION widen_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f64
local function rt_widen_f32(source)
	source = from_bits_f32(source)
	source = into_bits_f64(source)

	return source
end

-- SECTION saturate_f32_to_s32
-- NEEDS force_i32
-- NEEDS from_bits_f32
-- NEEDS math_modf
local function rt_saturate_f32_to_s32(source)
	source = from_bits_f32(source)

	if source >= 0x80000000 then
		source = force_i32(0x7FFFFFFF)
	elseif source <= -0x80000000 then
		source = force_i32(-0x80000000)
	elseif source ~= source then
		source = force_i32(0)
	else
		source = math_modf(source)
		source = force_i32(source)
	end

	return source
end

-- SECTION truncate_f32_to_s32
-- NEEDS force_i32
-- NEEDS from_bits_f32
-- NEEDS math_modf
local function rt_truncate_f32_to_s32(source)
	source = from_bits_f32(source)
	source = math_modf(source)

	if source >= 0x80000000 or source < -0x80000000 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = force_i32(source)

	return source
end

-- SECTION saturate_f32_to_u32
-- NEEDS force_i32
-- NEEDS from_bits_f32
-- NEEDS math_floor
local function rt_saturate_f32_to_u32(source)
	source = from_bits_f32(source)

	if source >= 0x100000000 then
		source = force_i32(0xFFFFFFFF)
	elseif source <= 0 or source ~= source then
		source = force_i32(0)
	else
		source = math_floor(source)
		source = force_i32(source)
	end

	return source
end

-- SECTION truncate_f32_to_u32
-- NEEDS force_i32
-- NEEDS from_bits_f32
-- NEEDS math_modf
local function rt_truncate_f32_to_u32(source)
	source = from_bits_f32(source)
	source = math_modf(source)

	if source >= 0x100000000 or source < 0 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = force_i32(source)

	return source
end

-- SECTION saturate_f32_to_s64
-- NEEDS ffi_cast
-- NEEDS from_bits_f32
-- NEEDS i64_type
-- NEEDS math_modf
local function rt_saturate_f32_to_s64(source)
	source = from_bits_f32(source)

	if source >= 0x8000000000000000 then
		source = 0x7FFFFFFFFFFFFFFFLL
	elseif source <= -0x8000000000000000 then
		source = -0x8000000000000000LL
	elseif source ~= source then
		source = 0LL
	else
		source = math_modf(source)
		source = ffi_cast(i64_type, source)
	end

	return source
end

-- SECTION truncate_f32_to_s64
-- NEEDS ffi_cast
-- NEEDS from_bits_f32
-- NEEDS i64_type
-- NEEDS math_modf
local function rt_truncate_f32_to_s64(source)
	source = from_bits_f32(source)
	source = math_modf(source)

	if source >= 0x8000000000000000 or source < -0x8000000000000000 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = ffi_cast(i64_type, source)

	return source
end

-- SECTION saturate_f32_to_u64
-- NEEDS ffi_cast
-- NEEDS from_bits_f32
-- NEEDS i64_type
-- NEEDS math_floor
-- NEEDS u64_type
local function rt_saturate_f32_to_u64(source)
	source = from_bits_f32(source)

	if source >= 0x10000000000000000 then
		source = 0xFFFFFFFFFFFFFFFFLL
	elseif source <= 0 or source ~= source then
		source = 0LL
	else
		source = math_floor(source)
		source = ffi_cast(u64_type, source)
		source = ffi_cast(i64_type, source)
	end

	return source
end

-- SECTION truncate_f32_to_u64
-- NEEDS ffi_cast
-- NEEDS from_bits_f32
-- NEEDS i64_type
-- NEEDS math_modf
-- NEEDS u64_type
local function rt_truncate_f32_to_u64(source)
	source = from_bits_f32(source)
	source = math_modf(source)

	if source >= 0x10000000000000000 or source < 0 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = ffi_cast(u64_type, source)
	source = ffi_cast(i64_type, source)

	return source
end

-- SECTION transmute_f32_to_i32
local function rt_transmute_f32_to_i32(source)
	return source
end
