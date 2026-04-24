-- SECTION transmute_n64
local TRANSMUTE_N64 = buffer.create(8)

-- SECTION buffer_read_i8
local buffer_read_i8 = buffer.readi8

-- SECTION buffer_read_u8
local buffer_read_u8 = buffer.readu8

-- SECTION buffer_read_i16
local buffer_read_i16 = buffer.readi16

-- SECTION buffer_read_u16
local buffer_read_u16 = buffer.readu16

-- SECTION buffer_read_i32
local buffer_read_i32 = buffer.readi32

-- SECTION buffer_read_u32
local buffer_read_u32 = buffer.readu32

-- SECTION buffer_read_f32
local buffer_read_f32 = buffer.readf32

-- SECTION buffer_read_f64
local buffer_read_f64 = buffer.readf64

-- SECTION buffer_write_u8
local buffer_write_u8 = buffer.writeu8

-- SECTION buffer_write_u16
local buffer_write_u16 = buffer.writeu16

-- SECTION buffer_write_u32
local buffer_write_u32 = buffer.writeu32

-- SECTION buffer_write_f32
local buffer_write_f32 = buffer.writef32

-- SECTION buffer_write_f64
local buffer_write_f64 = buffer.writef64

-- SECTION is_positive
-- NEEDS buffer_read_i8
-- NEEDS buffer_write_f64
-- NEEDS transmute_n64
local function is_positive(source: number): boolean
	buffer_write_f64(TRANSMUTE_N64, 0, source)

	return buffer_read_i8(TRANSMUTE_N64, 7) >= 0
end

-- SECTION flip_most_significant
-- NEEDS bit32_xor
-- NEEDS from_bits_i64
-- NEEDS into_bits_i64
local function flip_most_significant(source: number): number
	local source_1, source_2 = from_bits_i64(source)

	source_2 = bit32_xor(source_2, 0x8000_0000)
	source = into_bits_i64(source_1, source_2)

	return source
end

-- SECTION is_all_zeros
-- NEEDS buffer_read_u32
-- NEEDS buffer_write_f64
-- NEEDS transmute_n64
local function is_all_zeros(source: number): boolean
	buffer_write_f64(TRANSMUTE_N64, 0, source)

	return buffer_read_u32(TRANSMUTE_N64, 0) == 0 and buffer_read_u32(TRANSMUTE_N64, 4) == 0
end

-- SECTION from_bits_i64_to_i16
-- NEEDS buffer_read_u16
-- NEEDS buffer_write_f64
-- NEEDS transmute_n64
local function from_bits_i64_to_i16(source: number): (number, number, number, number)
	buffer_write_f64(TRANSMUTE_N64, 0, source)

	return buffer_read_u16(TRANSMUTE_N64, 0),
		buffer_read_u16(TRANSMUTE_N64, 2),
		buffer_read_u16(TRANSMUTE_N64, 4),
		buffer_read_u16(TRANSMUTE_N64, 6)
end

-- SECTION from_bits_i64
-- NEEDS buffer_read_u32
-- NEEDS buffer_write_f64
-- NEEDS transmute_n64
local function from_bits_i64(source: number): (number, number)
	buffer_write_f64(TRANSMUTE_N64, 0, source)

	return buffer_read_u32(TRANSMUTE_N64, 0), buffer_read_u32(TRANSMUTE_N64, 4)
end

-- SECTION into_bits_i64
-- NEEDS buffer_read_f64
-- NEEDS buffer_write_u32
-- NEEDS transmute_n64
local function into_bits_i64(source_1: number, source_2: number): number
	buffer_write_u32(TRANSMUTE_N64, 0, source_1)
	buffer_write_u32(TRANSMUTE_N64, 4, source_2)

	return buffer_read_f64(TRANSMUTE_N64, 0)
end

-- SECTION from_bits_f32
-- NEEDS buffer_read_f32
-- NEEDS buffer_write_u32
-- NEEDS transmute_n64
local function from_bits_f32(source: number): number
	buffer_write_u32(TRANSMUTE_N64, 0, source)

	return buffer_read_f32(TRANSMUTE_N64, 0)
end

-- SECTION into_bits_f32
-- NEEDS buffer_read_u32
-- NEEDS buffer_write_f32
-- NEEDS transmute_n64
local function into_bits_f32(source: number): number
	buffer_write_f32(TRANSMUTE_N64, 0, source)

	return buffer_read_u32(TRANSMUTE_N64, 0)
end
