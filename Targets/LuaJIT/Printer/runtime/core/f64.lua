-- SECTION absolute_f64
-- NEEDS bit_and
local function rt_absolute_f64(source)
	source = bit_and(source, 0x7FFFFFFFFFFFFFFFLL)

	return source
end

-- SECTION negate_f64
-- NEEDS bit_xor
local function rt_negate_f64(source)
	source = bit_xor(source, 0x8000000000000000LL)

	return source
end

-- SECTION square_root_f64
-- NEEDS from_bits_f64
-- NEEDS into_bits_f64
-- NEEDS math_sqrt
local function rt_square_root_f64(source)
	source = from_bits_f64(source)
	source = math_sqrt(source)
	source = into_bits_f64(source)

	return source
end

-- SECTION round_up_f64
-- NEEDS from_bits_f64
-- NEEDS into_bits_f64
-- NEEDS math_ceil
local function rt_round_up_f64(source)
	source = from_bits_f64(source)
	source = math_ceil(source)
	source = into_bits_f64(source)

	return source
end

-- SECTION round_down_f64
-- NEEDS from_bits_f64
-- NEEDS into_bits_f64
-- NEEDS math_floor
local function rt_round_down_f64(source)
	source = from_bits_f64(source)
	source = math_floor(source)
	source = into_bits_f64(source)

	return source
end

-- SECTION truncate_f64
-- NEEDS from_bits_f64
-- NEEDS into_bits_f64
-- NEEDS math_modf
local function rt_truncate_f64(source)
	source = from_bits_f64(source)
	source = math_modf(source)
	source = into_bits_f64(source)

	return source
end

-- SECTION nearest_f64
-- NEEDS from_bits_f64
-- NEEDS into_bits_f64
-- NEEDS math_abs
-- NEEDS math_modf
local function rt_nearest_f64(source)
	local native = from_bits_f64(source)

	native = math_abs(native)

	local rounded, remainder = math_modf(native)

	if remainder > 0.5 or (remainder == 0.5 and rounded % 2 == 1) then
		rounded = rounded + 1
	end

	if source < 0LL then
		rounded = -rounded
	end

	source = into_bits_f64(rounded)

	return source
end

-- SECTION add_f64
-- NEEDS from_bits_f64
-- NEEDS into_bits_f64
local function rt_add_f64(lhs, rhs)
	lhs = from_bits_f64(lhs)
	rhs = from_bits_f64(rhs)

	local result = into_bits_f64(lhs + rhs)

	return result
end

-- SECTION subtract_f64
-- NEEDS from_bits_f64
-- NEEDS into_bits_f64
local function rt_subtract_f64(lhs, rhs)
	lhs = from_bits_f64(lhs)
	rhs = from_bits_f64(rhs)

	local result = into_bits_f64(lhs - rhs)

	return result
end

-- SECTION multiply_f64
-- NEEDS from_bits_f64
-- NEEDS into_bits_f64
local function rt_multiply_f64(lhs, rhs)
	lhs = from_bits_f64(lhs)
	rhs = from_bits_f64(rhs)

	local result = into_bits_f64(lhs * rhs)

	return result
end

-- SECTION divide_f64
-- NEEDS from_bits_f64
-- NEEDS into_bits_f64
local function rt_divide_f64(lhs, rhs)
	lhs = from_bits_f64(lhs)
	rhs = from_bits_f64(rhs)

	local result = into_bits_f64(lhs / rhs)

	return result
end

-- SECTION minimum_f64
-- NEEDS from_bits_f64
-- NEEDS into_bits_f64
-- NEEDS math_min
local function rt_minimum_f64(lhs, rhs)
	if rhs >= 0LL then
		lhs, rhs = rhs, lhs
	end

	lhs = from_bits_f64(lhs)
	rhs = from_bits_f64(rhs)

	local result = math_min(lhs, rhs)

	result = into_bits_f64(result)

	return result
end

-- SECTION maximum_f64
-- NEEDS from_bits_f64
-- NEEDS into_bits_f64
-- NEEDS math_max
local function rt_maximum_f64(lhs, rhs)
	if rhs < 0LL then
		lhs, rhs = rhs, lhs
	end

	lhs = from_bits_f64(lhs)
	rhs = from_bits_f64(rhs)

	local result = math_max(lhs, rhs)

	result = into_bits_f64(result)

	return result
end

-- SECTION copy_sign_f64
-- NEEDS bit_and
-- NEEDS bit_or
local function rt_copy_sign_f64(lhs, rhs)
	lhs = bit_and(lhs, 0x7FFFFFFFFFFFFFFFLL)
	rhs = bit_and(rhs, 0x8000000000000000LL)

	local result = bit_or(lhs, rhs)

	return result
end

-- SECTION equal_f64
-- NEEDS from_bits_f64
local function rt_equal_f64(lhs, rhs)
	lhs = from_bits_f64(lhs)
	rhs = from_bits_f64(rhs)

	return lhs == rhs
end

-- SECTION not_equal_f64
-- NEEDS from_bits_f64
local function rt_not_equal_f64(lhs, rhs)
	lhs = from_bits_f64(lhs)
	rhs = from_bits_f64(rhs)

	return lhs ~= rhs
end

-- SECTION less_than_f64
-- NEEDS from_bits_f64
local function rt_less_than_f64(lhs, rhs)
	lhs = from_bits_f64(lhs)
	rhs = from_bits_f64(rhs)

	return lhs < rhs
end

-- SECTION less_than_equal_f64
-- NEEDS from_bits_f64
local function rt_less_than_equal_f64(lhs, rhs)
	lhs = from_bits_f64(lhs)
	rhs = from_bits_f64(rhs)

	return lhs <= rhs
end

-- SECTION narrow_f64
-- NEEDS from_bits_f64
-- NEEDS into_bits_f32
local function rt_narrow_f64(source)
	source = from_bits_f64(source)
	source = into_bits_f32(source)

	return source
end

-- SECTION saturate_f64_to_s32
-- NEEDS force_i32
-- NEEDS from_bits_f64
-- NEEDS math_modf
local function rt_saturate_f64_to_s32(source)
	source = from_bits_f64(source)

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

-- SECTION truncate_f64_to_s32
-- NEEDS force_i32
-- NEEDS from_bits_f64
-- NEEDS math_modf
local function rt_truncate_f64_to_s32(source)
	source = from_bits_f64(source)
	source = math_modf(source)

	if source >= 0x80000000 or source < -0x80000000 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = force_i32(source)

	return source
end

-- SECTION saturate_f64_to_u32
-- NEEDS force_i32
-- NEEDS from_bits_f64
-- NEEDS math_floor
local function rt_saturate_f64_to_u32(source)
	source = from_bits_f64(source)

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

-- SECTION truncate_f64_to_u32
-- NEEDS force_i32
-- NEEDS from_bits_f64
-- NEEDS math_modf
local function rt_truncate_f64_to_u32(source)
	source = from_bits_f64(source)
	source = math_modf(source)

	if source >= 0x100000000 or source < 0 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = force_i32(source)

	return source
end

-- SECTION saturate_f64_to_s64
-- NEEDS ffi_cast
-- NEEDS from_bits_f64
-- NEEDS i64_type
-- NEEDS math_modf
local function rt_saturate_f64_to_s64(source)
	source = from_bits_f64(source)

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

-- SECTION truncate_f64_to_s64
-- NEEDS ffi_cast
-- NEEDS from_bits_f64
-- NEEDS i64_type
-- NEEDS math_modf
local function rt_truncate_f64_to_s64(source)
	source = from_bits_f64(source)
	source = math_modf(source)

	if source >= 0x8000000000000000 or source < -0x8000000000000000 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = ffi_cast(i64_type, source)

	return source
end

-- SECTION saturate_f64_to_u64
-- NEEDS ffi_cast
-- NEEDS from_bits_f64
-- NEEDS i64_type
-- NEEDS math_floor
-- NEEDS u64_type
local function rt_saturate_f64_to_u64(source)
	source = from_bits_f64(source)

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

-- SECTION truncate_f64_to_u64
-- NEEDS ffi_cast
-- NEEDS from_bits_f64
-- NEEDS i64_type
-- NEEDS math_modf
-- NEEDS u64_type
local function rt_truncate_f64_to_u64(source)
	source = from_bits_f64(source)
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

-- SECTION transmute_f64_to_i64
local function rt_transmute_f64_to_i64(source)
	return source
end
