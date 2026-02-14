-- SECTION absolute_f64
local rt_absolute_f64 = math.abs

-- SECTION negate_f64
local function rt_negate_f64(source: number): number
	return -source
end

-- SECTION square_root_f64
local rt_square_root_f64 = math.sqrt

-- SECTION round_up_f64
local rt_round_up_f64 = math.ceil

-- SECTION round_down_f64
local rt_round_down_f64 = math.floor

-- SECTION truncate_f64
local rt_truncate_f64 = math.modf

-- SECTION nearest_f64
-- NEEDS is_positive
-- NEEDS math_abs
-- NEEDS math_modf
local function rt_nearest_f64(source: number): number
	local absolute = math_abs(source)
	local rounded, remainder = math_modf(absolute)

	if remainder > 0.5 or (remainder == 0.5 and rounded % 2 == 1) then
		rounded = rounded + 1
	end

	if not is_positive(source) then
		rounded = -rounded
	end

	return rounded
end

-- SECTION add_f64
local function rt_add_f64(lhs: number, rhs: number): number
	return lhs + rhs
end

-- SECTION subtract_f64
local function rt_subtract_f64(lhs: number, rhs: number): number
	return lhs - rhs
end

-- SECTION multiply_f64
local function rt_multiply_f64(lhs: number, rhs: number): number
	return lhs * rhs
end

-- SECTION divide_f64
local function rt_divide_f64(lhs: number, rhs: number): number
	return lhs / rhs
end

-- SECTION minimum_f64
-- NEEDS is_positive
-- NEEDS math_min
local function rt_minimum_f64(lhs: number, rhs: number): number
	if is_positive(lhs) then
		lhs, rhs = rhs, lhs
	end

	local result = math_min(lhs, rhs)

	return result
end

-- SECTION maximum_f64
-- NEEDS is_positive
-- NEEDS math_max
local function rt_maximum_f64(lhs: number, rhs: number): number
	if is_positive(rhs) then
		lhs, rhs = rhs, lhs
	end

	local result = math_max(lhs, rhs)

	return result
end

-- SECTION copy_sign_f64
-- NEEDS is_positive
-- NEEDS math_abs
local function rt_copy_sign_f64(lhs: number, rhs: number): number
	lhs = math_abs(lhs)

	if not is_positive(rhs) then
		lhs = -lhs
	end

	return lhs
end

-- SECTION equal_f64
local function rt_equal_f64(lhs: number, rhs: number): boolean
	return lhs == rhs
end

-- SECTION not_equal_f64
local function rt_not_equal_f64(lhs: number, rhs: number): boolean
	return lhs ~= rhs
end

-- SECTION less_than_f64
local function rt_less_than_f64(lhs: number, rhs: number): boolean
	return lhs < rhs
end

-- SECTION greater_than_f64
local function rt_greater_than_f64(lhs: number, rhs: number): boolean
	return lhs > rhs
end

-- SECTION less_than_equal_f64
local function rt_less_than_equal_f64(lhs: number, rhs: number): boolean
	return lhs <= rhs
end

-- SECTION greater_than_equal_f64
local function rt_greater_than_equal_f64(lhs: number, rhs: number): boolean
	return lhs >= rhs
end

-- SECTION narrow_f64
-- NEEDS buffer_read_u32
-- NEEDS buffer_write_f32
-- NEEDS transmute_n64
local function rt_narrow_f64(source: number): number
	buffer_write_f32(TRANSMUTE_N64, 0, source)

	return buffer_read_u32(TRANSMUTE_N64, 0)
end

-- SECTION saturate_f64_to_s32
-- NEEDS bit32_or
-- NEEDS math_modf
local function rt_saturate_f64_to_s32(source: number): number
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

-- SECTION truncate_f64_to_s32
-- NEEDS bit32_or
-- NEEDS math_modf
local function rt_truncate_f64_to_s32(source: number): number
	source = math_modf(source)

	if source >= 0x8000_0000 or source < -0x8000_0000 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = bit32_or(source, 0)

	return source
end

-- SECTION saturate_f64_to_u32
-- NEEDS bit32_or
-- NEEDS math_floor
local function rt_saturate_f64_to_u32(source: number): number
	if source >= 0x1_0000_0000 then
		source = 0xFFFF_FFFF
	elseif source <= 0 or source ~= source then
		source = 0
	else
		source = math_floor(source)
		source = bit32_or(source, 0)
	end

	return source
end

-- SECTION truncate_f64_to_u32
-- NEEDS bit32_or
-- NEEDS math_modf
local function rt_truncate_f64_to_u32(source: number): number
	source = math_modf(source)

	if source >= 0x100000000 or source < 0 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = bit32_or(source, 0)

	return source
end

-- SECTION saturate_f64_to_s64
-- NEEDS I64_0000_0000_0000_0000
-- NEEDS I64_7FFF_FFFF_FFFF_FFFF
-- NEEDS I64_8000_0000_0000_0000
-- NEEDS convert_f64_to_u64
-- NEEDS math_ceil
-- NEEDS math_floor
-- NEEDS negate_i64
local function rt_saturate_f64_to_s64(source: number): number
	if source >= 0x8000_0000_0000_0000 then
		source = I64_7FFF_FFFF_FFFF_FFFF
	elseif source <= -0x8000_0000_0000_0000 then
		source = I64_8000_0000_0000_0000
	elseif source ~= source then
		source = I64_0000_0000_0000_0000
	elseif source >= 0 then
		source = math_floor(source)
		source = convert_f64_to_u64(source)
	else
		source = math_ceil(source)
		source = convert_f64_to_u64(-source)
		source = negate_i64(source)
	end

	return source
end

-- SECTION truncate_f64_to_s64
-- NEEDS convert_f64_to_u64
-- NEEDS math_modf
-- NEEDS negate_i64
local function rt_truncate_f64_to_s64(source: number): number
	source = math_modf(source)

	if source >= 0x8000_0000_0000_0000 or source < -0x8000_0000_0000_0000 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	if source >= 0 then
		source = convert_f64_to_u64(source)
	else
		source = convert_f64_to_u64(-source)
		source = negate_i64(source)
	end

	return source
end

-- SECTION saturate_f64_to_u64
-- NEEDS I64_0000_0000_0000_0000
-- NEEDS I64_FFFF_FFFF_FFFF_FFFF
-- NEEDS convert_f64_to_u64
-- NEEDS math_floor
local function rt_saturate_f64_to_u64(source: number): number
	if source >= 0x1_0000_0000_0000_0000 then
		source = I64_FFFF_FFFF_FFFF_FFFF
	elseif source <= 0 or source ~= source then
		source = I64_0000_0000_0000_0000
	else
		source = math_floor(source)
		source = convert_f64_to_u64(source)
	end

	return source
end

-- SECTION truncate_f64_to_u64
-- NEEDS convert_f64_to_u64
-- NEEDS math_modf
local function rt_truncate_f64_to_u64(source: number): number
	source = math_modf(source)

	if source >= 0x1_0000_0000_0000_0000 or source < 0 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = convert_f64_to_u64(source)

	return source
end

-- SECTION transmute_f64_to_i64
local function rt_transmute_f64_to_i64(source: number): number
	return source
end
