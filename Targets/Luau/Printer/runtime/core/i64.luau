-- SECTION count_ones_i64
-- NEEDS count_ones_i32
-- NEEDS integer_create
-- NEEDS into_halves_i64
local function rt_count_ones_i64(source: integer): integer
	local source_1, source_2 = into_halves_i64(source)

	source_1 = rt_count_ones_i32(source_1)
	source_2 = rt_count_ones_i32(source_2)

	source = integer_create(source_1 + source_2)

	return source
end

-- SECTION leading_zeroes_i64
local rt_leading_zeroes_i64 = integer.countlz

-- SECTION trailing_zeroes_i64
local rt_trailing_zeroes_i64 = integer.countrz

-- SECTION add_i64
local rt_add_i64 = integer.add

-- SECTION subtract_i64
local rt_subtract_i64 = integer.sub

-- SECTION multiply_i64
local rt_multiply_i64 = integer.mul

-- SECTION divide_s64
local rt_divide_s64 = integer.div

-- SECTION divide_u64
local rt_divide_u64 = integer.udiv

-- SECTION remainder_s64
local rt_remainder_s64 = integer.rem

-- SECTION remainder_u64
local rt_remainder_u64 = integer.urem

-- SECTION and_i64
local rt_and_i64 = integer.band

-- SECTION or_i64
local rt_or_i64 = integer.bor

-- SECTION exclusive_or_i64
local rt_exclusive_or_i64 = integer.bxor

-- SECTION shift_left_i64
-- NEEDS integer_and
-- NEEDS integer_lshift
local function rt_shift_left_i64(lhs: integer, rhs: integer): integer
	rhs = integer_and(rhs, 0x3Fi)
	lhs = integer_lshift(lhs, rhs)

	return lhs
end

-- SECTION shift_right_s64
-- NEEDS integer_and
-- NEEDS integer_arshift
local function rt_shift_right_s64(lhs: integer, rhs: integer): integer
	rhs = integer_and(rhs, 0x3Fi)
	lhs = integer_arshift(lhs, rhs)

	return lhs
end

-- SECTION shift_right_u64
-- NEEDS integer_and
-- NEEDS integer_rshift
local function rt_shift_right_u64(lhs: integer, rhs: integer): integer
	rhs = integer_and(rhs, 0x3Fi)
	lhs = integer_rshift(lhs, rhs)

	return lhs
end

-- SECTION rotate_left_i64
local rt_rotate_left_i64 = integer.lrotate

-- SECTION rotate_right_i64
local rt_rotate_right_i64 = integer.rrotate

-- SECTION equal_i64
local function rt_equal_i64(lhs: integer, rhs: integer): boolean
	return lhs == rhs
end

-- SECTION not_equal_i64
local function rt_not_equal_i64(lhs: integer, rhs: integer): boolean
	return lhs ~= rhs
end

-- SECTION less_than_s64
local rt_less_than_s64 = integer.lt

-- SECTION less_than_u64
local rt_less_than_u64 = integer.ult

-- SECTION greater_than_s64
-- NEEDS integer_lt
local function rt_greater_than_s64(lhs: integer, rhs: integer): boolean
	return integer_lt(rhs, lhs)
end

-- SECTION greater_than_u64
-- NEEDS integer_ult
local function rt_greater_than_u64(lhs: integer, rhs: integer): boolean
	return integer_ult(rhs, lhs)
end

-- SECTION less_than_equal_s64
local rt_less_than_equal_s64 = integer.le

-- SECTION less_than_equal_u64
local rt_less_than_equal_u64 = integer.ule

-- SECTION greater_than_equal_s64
-- NEEDS integer_le
local function rt_greater_than_equal_s64(lhs: integer, rhs: integer): boolean
	return integer_le(rhs, lhs)
end

-- SECTION greater_than_equal_u64
-- NEEDS integer_ule
local function rt_greater_than_equal_u64(lhs: integer, rhs: integer): boolean
	return integer_ule(rhs, lhs)
end

-- SECTION narrow_i64
-- NEEDS into_halves_i64
local function rt_narrow_i64(source: integer): number
	source = into_halves_i64(source)

	return source
end

-- SECTION extend_s8_to_i64
-- NEEDS integer_and
-- NEEDS integer_lt
-- NEEDS integer_sub
local function rt_extend_s8_to_i64(source: integer): integer
	source = integer_and(source, 0xFFi)

	if not integer_lt(source, 0x80i) then
		source = integer_sub(source, 0x100i)
	end

	return source
end

-- SECTION extend_s16_to_i64
-- NEEDS integer_and
-- NEEDS integer_lt
-- NEEDS integer_sub
local function rt_extend_s16_to_i64(source: integer): integer
	source = integer_and(source, 0xFFFFi)

	if not integer_lt(source, 0x8000i) then
		source = integer_sub(source, 0x1_0000i)
	end

	return source
end

-- SECTION extend_s32_to_i64
-- NEEDS integer_and
-- NEEDS integer_lt
-- NEEDS integer_sub
local function rt_extend_s32_to_i64(source: integer): integer
	source = integer_and(source, 0xFFFF_FFFFi)

	if not integer_lt(source, 0x8000_0000i) then
		source = integer_sub(source, 0x1_0000_0000i)
	end

	return source
end

-- SECTION convert_s64_to_f32
-- NEEDS integer_tonumber
-- NEEDS narrow_f64
local function rt_convert_s64_to_f32(source: integer): number
	source = integer_tonumber(source)
	source = rt_narrow_f64(source)

	return source
end

-- SECTION convert_u64_to_f32
-- NEEDS into_halves_i64
-- NEEDS narrow_f64
local function rt_convert_u64_to_f32(source: integer): number
	local source_1, source_2 = into_halves_i64(source)

	source = source_1 + source_2 * 0x1_0000_0000
	source = rt_narrow_f64(source)

	return source
end

-- SECTION convert_s64_to_f64
local rt_convert_s64_to_f64 = integer.tonumber

-- SECTION convert_u64_to_f64
-- NEEDS into_halves_i64
local function rt_convert_u64_to_f64(source: integer): number
	local source_1, source_2 = into_halves_i64(source)

	source = source_1 + source_2 * 0x1_0000_0000

	return source
end

-- SECTION transmute_i64_to_f64
-- NEEDS buffer_read_f64
-- NEEDS buffer_write_integer
-- NEEDS transmute_n64
local function rt_transmute_i64_to_f64(source: integer): number
	buffer_write_integer(TRANSMUTE_N64, 0, source)

	source = buffer_read_f64(TRANSMUTE_N64, 0)

	return source
end
