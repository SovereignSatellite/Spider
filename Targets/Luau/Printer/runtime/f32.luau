-- SECTION absolute_f32
local rt_absolute_f32 = vector.abs

-- SECTION negate_f32
local function rt_negate_f32(source: vector): vector
	return -source
end

-- SECTION square_root_f32
-- NEEDS math_sqrt
-- NEEDS vector_create
local function rt_square_root_f32(source: vector): vector
	local source = math_sqrt(source.x)

	return vector_create(source, 0, 0)
end

-- SECTION round_up_f32
-- NEEDS math_ceil
-- NEEDS vector_create
local function rt_round_up_f32(source: vector): vector
	local source = math_ceil(source.x)

	return vector_create(source, 0, 0)
end

-- SECTION round_down_f32
-- NEEDS math_floor
-- NEEDS vector_create
local function rt_round_down_f32(source: vector): vector
	local source = math_floor(source.x)

	return vector_create(source, 0, 0)
end

-- SECTION truncate_f32
-- NEEDS math_ceil
-- NEEDS math_floor
-- NEEDS vector_create
local function rt_truncate_f32(source: vector): vector
	local source = if source.x >= 0 then math_floor(source.x) else math_ceil(source.x)

	return vector_create(source, 0, 0)
end

-- SECTION nearest_f32
-- NEEDS is_positive
-- NEEDS math_abs
-- NEEDS math_round
-- NEEDS vector_create
local function rt_nearest_f32(source: vector): vector
	local positive = is_positive(source.x)
	local source = math_abs(source.x)
	local rounded = math_round(source)

	if source == rounded - 0.5 and rounded % 2 == 1 then
		rounded = rounded - 1
	end

	return vector_create(if positive then rounded else -rounded, 0, 0)
end

-- SECTION add_f32
local function rt_add_f32(lhs: vector, rhs: vector): vector
	return lhs + rhs
end

-- SECTION subtract_f32
local function rt_subtract_f32(lhs: vector, rhs: vector): vector
	return lhs - rhs
end

-- SECTION multiply_f32
local function rt_multiply_f32(lhs: vector, rhs: vector): vector
	return lhs * rhs
end

-- SECTION divide_f32
local function rt_divide_f32(lhs: vector, rhs: vector): vector
	return lhs / rhs
end

-- SECTION minimum_f32
-- NEEDS is_positive
-- NEEDS vector_min
local function rt_minimum_f32(lhs: vector, rhs: vector): vector
	return if is_positive(rhs.x) then vector_min(lhs, rhs) else vector_min(rhs, lhs)
end

-- SECTION maximum_f32
-- NEEDS is_positive
-- NEEDS vector_max
local function rt_maximum_f32(lhs: vector, rhs: vector): vector
	return if is_positive(lhs.x) then vector_max(lhs, rhs) else vector_max(rhs, lhs)
end

-- SECTION copy_sign_f32
-- NEEDS is_positive
-- NEEDS vector_abs
local function rt_copy_sign_f32(lhs: vector, rhs: vector): vector
	return if is_positive(rhs.x) then vector_abs(lhs) else -vector_abs(lhs)
end

-- SECTION equal_f32
local function rt_equal_f32(lhs: vector, rhs: vector): number
	return if lhs.x == rhs.x then 1 else 0
end

-- SECTION not_equal_f32
local function rt_not_equal_f32(lhs: vector, rhs: vector): number
	return if lhs.x == rhs.x then 0 else 1
end

-- SECTION less_than_f32
local function rt_less_than_f32(lhs: vector, rhs: vector): number
	return if lhs.x < rhs.x then 1 else 0
end

-- SECTION greater_than_f32
local function rt_greater_than_f32(lhs: vector, rhs: vector): number
	return if lhs.x > rhs.x then 1 else 0
end

-- SECTION less_than_equal_f32
local function rt_less_than_equal_f32(lhs: vector, rhs: vector): number
	return if lhs.x <= rhs.x then 1 else 0
end

-- SECTION greater_than_equal_f32
local function rt_greater_than_equal_f32(lhs: vector, rhs: vector): number
	return if lhs.x >= rhs.x then 1 else 0
end

-- SECTION widen_f32
local function rt_widen_f32(source: vector): number
	return source.x
end

-- SECTION saturate_f32_to_s32
-- NEEDS saturate_f64_to_s32
local function rt_saturate_f32_to_s32(source: vector): vector
	return rt_saturate_f64_to_s32(source.x)
end

-- SECTION truncate_f32_to_s32
-- NEEDS truncate_f64_to_s32
local function rt_truncate_f32_to_s32(source: vector): number
	return rt_truncate_f64_to_s32(source.x)
end

-- SECTION saturate_f32_to_u32
-- NEEDS saturate_f64_to_u32
local function rt_saturate_f32_to_u32(source: vector): vector
	return rt_saturate_f64_to_u32(source.x)
end

-- SECTION truncate_f32_to_u32
-- NEEDS truncate_f64_to_u32
local function rt_truncate_f32_to_u32(source: vector): number
	return rt_truncate_f64_to_u32(source.x)
end

-- SECTION saturate_f32_to_s64
-- NEEDS saturate_f64_to_s64
local function rt_saturate_f32_to_s64(source: vector): number
	return rt_saturate_f64_to_s64(source.x)
end

-- SECTION truncate_f32_to_s64
-- NEEDS truncate_f64_to_s64
local function rt_truncate_f32_to_s64(source: vector): number
	return rt_truncate_f64_to_s64(source.x)
end

-- SECTION saturate_f32_to_u64
-- NEEDS saturate_f64_to_u64
local function rt_saturate_f32_to_u64(source: vector): number
	return rt_saturate_f64_to_u64(source.x)
end

-- SECTION truncate_f32_to_u64
-- NEEDS truncate_f64_to_u64
local function rt_truncate_f32_to_u64(source: vector): number
	return rt_truncate_f64_to_u64(source.x)
end

-- SECTION transmute_f32_to_i32
-- NEEDS buffer_read_u32
-- NEEDS buffer_write_f32
-- NEEDS raw_memory
local function rt_transmute_f32_to_i32(source: vector): number
	buffer_write_f32(RAW_MEMORY, 0, source.x)

	return buffer_read_u32(RAW_MEMORY, 0)
end
