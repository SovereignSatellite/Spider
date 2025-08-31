-- SECTION create_f64_from_u32
-- NEEDS buffer_read_f64
-- NEEDS buffer_write_u32
-- NEEDS raw_memory
local function rt_create_f64_from_u32(source_1: number, source_2: number): number
	buffer_write_u32(RAW_MEMORY, 0, source_1)
	buffer_write_u32(RAW_MEMORY, 4, source_2)

	return buffer_read_f64(RAW_MEMORY, 0)
end

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
-- NEEDS math_ceil
-- NEEDS math_floor
local function rt_truncate_f64(source: number): number
	return if source >= 0 then math_floor(source) else math_ceil(source)
end

-- SECTION nearest_f64
-- NEEDS is_positive
-- NEEDS math_abs
-- NEEDS math_round
local function rt_nearest_f64(source: number): number
	local positive = is_positive(source)
	local source = math_abs(source)
	local rounded = math_round(source)

	if source == rounded - 0.5 and rounded % 2 == 1 then
		rounded = rounded - 1
	end

	return if positive then rounded else -rounded
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
	return if is_positive(rhs) then math_min(lhs, rhs) else math_min(rhs, lhs)
end

-- SECTION maximum_f64
-- NEEDS is_positive
-- NEEDS math_max
local function rt_maximum_f64(lhs: number, rhs: number): number
	return if is_positive(lhs) then math_max(lhs, rhs) else math_max(rhs, lhs)
end

-- SECTION copy_sign_f64
-- NEEDS is_positive
-- NEEDS math_abs
local function rt_copy_sign_f64(lhs: number, rhs: number): number
	return if is_positive(rhs) then math_abs(lhs) else -math_abs(lhs)
end

-- SECTION equal_f64
local function rt_equal_f64(lhs: number, rhs: number): number
	return if lhs == rhs then 1 else 0
end

-- SECTION not_equal_f64
local function rt_not_equal_f64(lhs: number, rhs: number): number
	return if lhs == rhs then 0 else 1
end

-- SECTION less_than_f64
local function rt_less_than_f64(lhs: number, rhs: number): number
	return if lhs < rhs then 1 else 0
end

-- SECTION greater_than_f64
local function rt_greater_than_f64(lhs: number, rhs: number): number
	return if lhs > rhs then 1 else 0
end

-- SECTION less_than_equal_f64
local function rt_less_than_equal_f64(lhs: number, rhs: number): number
	return if lhs <= rhs then 1 else 0
end

-- SECTION greater_than_equal_f64
local function rt_greater_than_equal_f64(lhs: number, rhs: number): number
	return if lhs >= rhs then 1 else 0
end

-- SECTION narrow_f64
-- NEEDS vector_create
local function rt_narrow_f64(source: number): vector
	return vector_create(source, 0, 0)
end

-- SECTION saturate_f64_to_s32
-- NEEDS bit_or
-- NEEDS math_ceil
-- NEEDS math_floor
local function rt_saturate_f64_to_s32(source: number): number
	if source ~= source then
		return 0
	end

	if source >= 0 then
		source = math_floor(source)

		if source >= 0x8000_0000 then
			return 0x7FFF_FFFF
		end
	else
		source = math_ceil(source)

		if source < -0x8000_0000 then
			return 0x8000_0000
		end
	end

	return bit_or(source, 0)
end

-- SECTION truncate_f64_to_s32
-- NEEDS bit_or
-- NEEDS math_ceil
-- NEEDS math_floor
local function rt_truncate_f64_to_s32(source: number): number
	if source ~= source then
		error("invalid conversion to integer", 2)
	end

	if source >= 0 then
		source = math_floor(source)

		if source >= 0x8000_0000 then
			error("integer overflow", 2)
		end
	else
		source = math_ceil(source)

		if source < -0x8000_0000 then
			error("integer overflow", 2)
		end
	end

	return bit_or(source, 0)
end

-- SECTION saturate_f64_to_u32
-- NEEDS bit_or
-- NEEDS math_ceil
-- NEEDS math_floor
local function rt_saturate_f64_to_u32(source: number): number
	if source ~= source then
		return 0
	end

	if source >= 0 then
		source = math_floor(source)

		if source >= 0x1_0000_0000 then
			return 0xFFFF_FFFF
		end
	else
		source = math_ceil(source)

		if source < 0 then
			return 0
		end
	end

	return bit_or(source, 0)
end

-- SECTION truncate_f64_to_u32
-- NEEDS bit_or
-- NEEDS math_ceil
-- NEEDS math_floor
local function rt_truncate_f64_to_u32(source: number): number
	if source ~= source then
		error("invalid conversion to integer", 2)
	end

	if source >= 0 then
		source = math_floor(source)

		if source >= 0x1_0000_0000 then
			error("integer overflow", 2)
		end
	else
		source = math_ceil(source)

		if source < 0 then
			error("integer overflow", 2)
		end
	end

	return bit_or(source, 0)
end

-- SECTION saturate_f64_to_s64
-- NEEDS constant_maximum_s64
-- NEEDS constant_minimum_s64
-- NEEDS constant_zero_u64
-- NEEDS math_ceil
-- NEEDS math_floor
-- NEEDS truncate_f64_to_u64_unchecked
local function rt_saturate_f64_to_s64(source: number): number
	if source ~= source then
		return ZERO_U64
	end

	if source >= 0 then
		local source = math_floor(source)

		if source >= 0x8000_0000_0000_0000 then
			return MAXIMUM_S64
		end

		return rt_truncate_f64_to_u64_unchecked(source)
	else
		local source = math_ceil(source)

		if source < -0x8000_0000_0000_0000 then
			return MINIMUM_S64
		end

		local source = rt_truncate_f64_to_u64_unchecked(-source)

		return rt_negate_i64(source)
	end
end

-- SECTION truncate_f64_to_s64
-- NEEDS math_ceil
-- NEEDS math_floor
-- NEEDS negate_i64
-- NEEDS truncate_f64_to_u64_unchecked
local function rt_truncate_f64_to_s64(source: number): number
	if source ~= source then
		error("invalid conversion to integer", 2)
	end

	if source >= 0 then
		local source = math_floor(source)

		if source >= 0x8000_0000_0000_0000 then
			error("integer overflow", 2)
		end

		return rt_truncate_f64_to_u64_unchecked(source)
	else
		local source = math_ceil(source)

		if source < -0x8000_0000_0000_0000 then
			error("integer overflow", 2)
		end

		local source = rt_truncate_f64_to_u64_unchecked(-source)

		return rt_negate_i64(source)
	end
end

-- SECTION saturate_f64_to_u64
-- NEEDS constant_all_u64
-- NEEDS constant_zero_u64
-- NEEDS math_ceil
-- NEEDS math_floor
-- NEEDS truncate_f64_to_u64_unchecked
local function rt_saturate_f64_to_u64(source: number): number
	if source ~= source then
		return ZERO_U64
	end

	if source >= 0 then
		source = math_floor(source)

		if source >= 0x1_0000_0000_0000_0000 then
			return ALL_U64
		end
	else
		source = math_ceil(source)

		if source < 0 then
			return ZERO_U64
		end
	end

	return rt_truncate_f64_to_u64_unchecked(source)
end

-- SECTION truncate_f64_to_u64_unchecked
-- NEEDS bit_or
-- NEEDS create_i64_from_u32
local function rt_truncate_f64_to_u64_unchecked(source: number): number
	local source_1 = bit_or(source % 0x1_0000_0000, 0)
	local source_2 = bit_or(source / 0x1_0000_0000, 0)

	return rt_create_i64_from_u32(source_1, source_2)
end

-- SECTION truncate_f64_to_u64
-- NEEDS math_ceil
-- NEEDS math_floor
-- NEEDS truncate_f64_to_u64_unchecked
local function rt_truncate_f64_to_u64(source: number): number
	if source ~= source then
		error("invalid conversion to integer", 2)
	end

	if source >= 0 then
		source = math_floor(source)

		if source >= 0x1_0000_0000_0000_0000 then
			error("integer overflow", 2)
		end
	else
		source = math_ceil(source)

		if source < 0 then
			error("integer overflow", 2)
		end
	end

	return rt_truncate_f64_to_u64_unchecked(source)
end

-- SECTION transmute_f64_to_i64
local function rt_transmute_f64_to_i64(source: number): number
	return source
end
