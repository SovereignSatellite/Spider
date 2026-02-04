-- SECTION absolute_f32
-- NEEDS bit_and
local function rt_absolute_f32(source: number): number
	return bit_and(source, 0x7FFF_FFFF)
end

-- SECTION negate_f32
-- NEEDS bit_xor
local function rt_negate_f32(source: number): number
	return bit_xor(source, 0x8000_0000)
end

-- SECTION square_root_f32
-- NEEDS math_sqrt
-- NEEDS narrow_f64
-- NEEDS widen_f32
local function rt_square_root_f32(source: number): number
	local source = rt_widen_f32(source)

	return rt_narrow_f64(math_sqrt(source))
end

-- SECTION round_up_f32
-- NEEDS math_ceil
-- NEEDS narrow_f64
-- NEEDS widen_f32
local function rt_round_up_f32(source: number): number
	local source = rt_widen_f32(source)

	return rt_narrow_f64(math_ceil(source))
end

-- SECTION round_down_f32
-- NEEDS math_floor
-- NEEDS narrow_f64
-- NEEDS widen_f32
local function rt_round_down_f32(source: number): number
	local source = rt_widen_f32(source)

	return rt_narrow_f64(math_floor(source))
end

-- SECTION truncate_f32
-- NEEDS math_ceil
-- NEEDS math_floor
-- NEEDS narrow_f64
-- NEEDS widen_f32
local function rt_truncate_f32(source: number): number
	local source = rt_widen_f32(source)

	return rt_narrow_f64(if source >= 0 then math_floor(source) else math_ceil(source))
end

-- SECTION nearest_f32
-- NEEDS bit_and
-- NEEDS math_abs
-- NEEDS math_round
-- NEEDS narrow_f64
-- NEEDS widen_f32
local function rt_nearest_f32(source: number): number
	local sign = bit_and(source, 0x80000000)
	local source = rt_widen_f32(source)
	local absolute = math_abs(source)
	local rounded = math_round(absolute)

	if absolute == rounded - 0.5 and rounded % 2 == 1 then
		rounded = rounded - 1
	end

	return rt_narrow_f64(if sign == 0 then rounded else -rounded)
end

-- SECTION add_f32
-- NEEDS narrow_f64
-- NEEDS vector_create
-- NEEDS widen_f32
local function rt_add_f32(lhs: number, rhs: number): number
	local lhs = vector_create(rt_widen_f32(lhs), 0, 0)
	local rhs = vector_create(rt_widen_f32(rhs), 0, 0)

	return rt_narrow_f64((lhs + rhs).x)
end

-- SECTION subtract_f32
-- NEEDS narrow_f64
-- NEEDS vector_create
-- NEEDS widen_f32
local function rt_subtract_f32(lhs: number, rhs: number): number
	local lhs = vector_create(rt_widen_f32(lhs), 0, 0)
	local rhs = vector_create(rt_widen_f32(rhs), 0, 0)

	return rt_narrow_f64((lhs - rhs).x)
end

-- SECTION multiply_f32
-- NEEDS narrow_f64
-- NEEDS vector_create
-- NEEDS widen_f32
local function rt_multiply_f32(lhs: number, rhs: number): number
	local lhs = vector_create(rt_widen_f32(lhs), 0, 0)
	local rhs = vector_create(rt_widen_f32(rhs), 0, 0)

	return rt_narrow_f64((lhs * rhs).x)
end

-- SECTION divide_f32
-- NEEDS narrow_f64
-- NEEDS vector_create
-- NEEDS widen_f32
local function rt_divide_f32(lhs: number, rhs: number): number
	local lhs = vector_create(rt_widen_f32(lhs), 0, 0)
	local rhs = vector_create(rt_widen_f32(rhs), 0, 0)

	return rt_narrow_f64((lhs / rhs).x)
end

-- SECTION minimum_f32
-- NEEDS bit_and
-- NEEDS math_min
-- NEEDS narrow_f64
-- NEEDS widen_f32
local function rt_minimum_f32(lhs: number, rhs: number): number
	local sign = bit_and(rhs, 0x8000_0000)
	local lhs = rt_widen_f32(lhs)
	local rhs = rt_widen_f32(rhs)

	return rt_narrow_f64(if sign == 0 then math_min(lhs, rhs) else math_min(rhs, lhs))
end

-- SECTION maximum_f32
-- NEEDS bit_and
-- NEEDS math_max
-- NEEDS narrow_f64
-- NEEDS widen_f32
local function rt_maximum_f32(lhs: number, rhs: number): number
	local sign = bit_and(lhs, 0x8000_0000)
	local lhs = rt_widen_f32(lhs)
	local rhs = rt_widen_f32(rhs)

	return rt_narrow_f64(if sign == 0 then math_max(lhs, rhs) else math_max(rhs, lhs))
end

-- SECTION copy_sign_f32
-- NEEDS bit_and
-- NEEDS bit_or
local function rt_copy_sign_f32(lhs: number, rhs: number): number
	return bit_or(bit_and(lhs, 0x7FFF_FFFF), bit_and(rhs, 0x8000_0000))
end

-- SECTION equal_f32
-- NEEDS widen_f32
local function rt_equal_f32(lhs: number, rhs: number): boolean
	return rt_widen_f32(lhs) == rt_widen_f32(rhs)
end

-- SECTION not_equal_f32
-- NEEDS widen_f32
local function rt_not_equal_f32(lhs: number, rhs: number): boolean
	return rt_widen_f32(lhs) ~= rt_widen_f32(rhs)
end

-- SECTION less_than_f32
-- NEEDS widen_f32
local function rt_less_than_f32(lhs: number, rhs: number): boolean
	return rt_widen_f32(lhs) < rt_widen_f32(rhs)
end

-- SECTION greater_than_f32
-- NEEDS widen_f32
local function rt_greater_than_f32(lhs: number, rhs: number): boolean
	return rt_widen_f32(lhs) > rt_widen_f32(rhs)
end

-- SECTION less_than_equal_f32
-- NEEDS widen_f32
local function rt_less_than_equal_f32(lhs: number, rhs: number): boolean
	return rt_widen_f32(lhs) <= rt_widen_f32(rhs)
end

-- SECTION greater_than_equal_f32
-- NEEDS widen_f32
local function rt_greater_than_equal_f32(lhs: number, rhs: number): boolean
	return rt_widen_f32(lhs) >= rt_widen_f32(rhs)
end

-- SECTION widen_f32
-- NEEDS buffer_read_f32
-- NEEDS buffer_write_u32
-- NEEDS raw_memory
local function rt_widen_f32(source: number): number
	buffer_write_u32(RAW_MEMORY, 0, source)

	return buffer_read_f32(RAW_MEMORY, 0)
end

-- SECTION saturate_f32_to_s32
-- NEEDS saturate_f64_to_s32
-- NEEDS widen_f32
local function rt_saturate_f32_to_s32(source: number): number
	local source = rt_widen_f32(source)

	return rt_saturate_f64_to_s32(source)
end

-- SECTION truncate_f32_to_s32
-- NEEDS truncate_f64_to_s32
-- NEEDS widen_f32
local function rt_truncate_f32_to_s32(source: number): number
	local source = rt_widen_f32(source)

	return rt_truncate_f64_to_s32(source)
end

-- SECTION saturate_f32_to_u32
-- NEEDS saturate_f64_to_u32
-- NEEDS widen_f32
local function rt_saturate_f32_to_u32(source: number): number
	local source = rt_widen_f32(source)

	return rt_saturate_f64_to_u32(source)
end

-- SECTION truncate_f32_to_u32
-- NEEDS truncate_f64_to_u32
-- NEEDS widen_f32
local function rt_truncate_f32_to_u32(source: number): number
	local source = rt_widen_f32(source)

	return rt_truncate_f64_to_u32(source)
end

-- SECTION saturate_f32_to_s64
-- NEEDS saturate_f64_to_s64
-- NEEDS widen_f32
local function rt_saturate_f32_to_s64(source: number): number
	local source = rt_widen_f32(source)

	return rt_saturate_f64_to_s64(source)
end

-- SECTION truncate_f32_to_s64
-- NEEDS truncate_f64_to_s64
-- NEEDS widen_f32
local function rt_truncate_f32_to_s64(source: number): number
	local source = rt_widen_f32(source)

	return rt_truncate_f64_to_s64(source)
end

-- SECTION saturate_f32_to_u64
-- NEEDS saturate_f64_to_u64
-- NEEDS widen_f32
local function rt_saturate_f32_to_u64(source: number): number
	local source = rt_widen_f32(source)

	return rt_saturate_f64_to_u64(source)
end

-- SECTION truncate_f32_to_u64
-- NEEDS truncate_f64_to_u64
-- NEEDS widen_f32
local function rt_truncate_f32_to_u64(source: number): number
	local source = rt_widen_f32(source)

	return rt_truncate_f64_to_u64(source)
end

-- SECTION transmute_f32_to_i32
local function rt_transmute_f32_to_i32(source: number): number
	return source
end
