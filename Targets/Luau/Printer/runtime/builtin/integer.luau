-- SECTION I64_0000_0000_0000_0000
-- NEEDS into_bits_i64
local I64_0000_0000_0000_0000 = into_bits_i64(0, 0)

-- SECTION I64_0000_0000_0000_0001
-- NEEDS into_bits_i64
local I64_0000_0000_0000_0001 = into_bits_i64(1, 0)

-- SECTION I64_0000_0000_0000_0040
-- NEEDS into_bits_i64
local I64_0000_0000_0000_0040 = into_bits_i64(0x40, 0)

-- SECTION I64_7FFF_FFFF_FFFF_FFFF
-- NEEDS into_bits_i64
local I64_7FFF_FFFF_FFFF_FFFF = into_bits_i64(0xFFFF_FFFF, 0x7FFF_FFFF)

-- SECTION I64_8000_0000_0000_0000
-- NEEDS into_bits_i64
local I64_8000_0000_0000_0000 = into_bits_i64(0x0000_0000, 0x8000_0000)

-- SECTION I64_FFFF_FFFF_FFFF_FFFF
-- NEEDS into_bits_i64
local I64_FFFF_FFFF_FFFF_FFFF = into_bits_i64(0xFFFF_FFFF, 0xFFFF_FFFF)

-- SECTION convert_f64_to_u64
-- NEEDS bit32_or
-- NEEDS into_bits_i64
local function convert_f64_to_u64(source: number): number
	local source_1 = bit32_or(source % 0x1_0000_0000, 0)
	local source_2 = bit32_or(source / 0x1_0000_0000, 0)

	source = into_bits_i64(source_1, source_2)

	return source
end

-- SECTION raw_leading_zeros_i64
-- NEEDS bit32_countlz
-- NEEDS buffer_read_u32
-- NEEDS buffer_write_f64
-- NEEDS transmute_n64
local function raw_leading_zeros_i64(source: number): number
	buffer_write_f64(TRANSMUTE_N64, 0, source)

	source = buffer_read_u32(TRANSMUTE_N64, 4)

	if source == 0 then
		source = buffer_read_u32(TRANSMUTE_N64, 0)
		source = bit32_countlz(source) + 32
	else
		source = bit32_countlz(source)
	end

	return source
end

-- SECTION raw_trailing_zeros_i64
-- NEEDS bit32_countrz
-- NEEDS buffer_read_u32
-- NEEDS buffer_write_f64
-- NEEDS transmute_n64
local function raw_trailing_zeros_i64(source: number): number
	buffer_write_f64(TRANSMUTE_N64, 0, source)

	source = buffer_read_u32(TRANSMUTE_N64, 0)

	if source == 0 then
		source = buffer_read_u32(TRANSMUTE_N64, 4)
		source = bit32_countrz(source) + 32
	else
		source = bit32_countrz(source)
	end

	return source
end

-- SECTION negate_i64
-- NEEDS bit32_not
-- NEEDS from_bits_i64
-- NEEDS into_bits_i64
local function negate_i64(source: number): number
	local source_1, source_2 = from_bits_i64(source)

	source_1 = bit32_not(source_1) + 1
	source_2 = bit32_not(source_2)

	if source_1 >= 0x1_0000_0000 then
		source_1 = source_1 - 0x1_0000_0000
		source_2 = source_2 + 1
	end

	if source_2 >= 0x1_0000_0000 then
		source_2 = source_2 - 0x1_0000_0000
	end

	source = into_bits_i64(source_1, source_2)

	return source
end

-- SECTION multiply_fast_i64
-- NEEDS convert_f64_to_u64
-- NEEDS from_bits_i64
local function multiply_fast_i64(lhs: number, rhs: number): number?
	local lhs_1, lhs_2 = from_bits_i64(lhs)
	local rhs_1, rhs_2 = from_bits_i64(rhs)

	return if lhs_2 == 0
			and rhs_2 == 0
			and lhs_1 <= 0x400_0000
			and rhs_1 <= 0x400_0000
		then convert_f64_to_u64(lhs_1 * rhs_1)
		else nil
end

-- SECTION raw_shift_left_i64
-- NEEDS bit32_lshift
-- NEEDS bit32_rshift
-- NEEDS from_bits_i64
-- NEEDS into_bits_i64
local function raw_shift_left_i64(lhs: number, rhs: number): number
	local source_1, source_2 = from_bits_i64(lhs)

	if rhs >= 32 then
		rhs = bit32_lshift(source_1, rhs - 32)
		lhs = 0
	else
		lhs = bit32_lshift(source_1, rhs)
		rhs = bit32_or(bit32_lshift(source_2, rhs), bit32_rshift(source_1, 32 - rhs))
	end

	local result = into_bits_i64(lhs, rhs)

	return result
end

-- SECTION raw_shift_right_s64
-- NEEDS bit32_arshift
-- NEEDS bit32_lshift
-- NEEDS bit32_rshift
-- NEEDS from_bits_i64
-- NEEDS into_bits_i64
local function raw_shift_right_s64(lhs: number, rhs: number): number
	local source_1, source_2 = from_bits_i64(lhs)

	if rhs >= 32 then
		lhs = bit32_arshift(source_2, rhs - 32)
		rhs = if source_2 < 0x8000_0000 then 0 else 0xFFFF_FFFF
	else
		lhs = bit32_or(bit32_rshift(source_1, rhs), bit32_lshift(source_2, 32 - rhs))
		rhs = bit32_arshift(source_2, rhs)
	end

	local result = into_bits_i64(lhs, rhs)

	return result
end

-- SECTION raw_shift_right_u64
-- NEEDS bit32_lshift
-- NEEDS bit32_rshift
-- NEEDS from_bits_i64
-- NEEDS into_bits_i64
local function raw_shift_right_u64(lhs: number, rhs: number): number
	local source_1, source_2 = from_bits_i64(lhs)

	if rhs >= 32 then
		lhs = bit32_rshift(source_2, rhs - 32)
		rhs = 0
	else
		lhs = bit32_or(bit32_rshift(source_1, rhs), bit32_lshift(source_2, 32 - rhs))
		rhs = bit32_rshift(source_2, rhs)
	end

	local result = into_bits_i64(lhs, rhs)

	return result
end
