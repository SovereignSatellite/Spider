-- SECTION memory_new
local function rt_memory_new(initializer: { string }, size: number): buffer?
	local ok, data = pcall(buffer.create, size)

	if not ok then
		return nil
	end

	for offset, content in initializer do
		buffer.writestring(data, offset, content)
	end

	return data
end

-- SECTION load_i32_from_s8
-- NEEDS bit32_or
-- NEEDS buffer_read_i8
local function rt_load_i32_from_s8(source: buffer, offset: number): number
	local result = buffer_read_i8(source, offset)

	result = bit32_or(result, 0)

	return result
end

-- SECTION load_i32_from_u8
-- NEEDS buffer_read_u8
local rt_load_i32_from_u8 = buffer_read_u8

-- SECTION load_i32_from_s16
-- NEEDS bit32_or
-- NEEDS buffer_read_i16
local function rt_load_i32_from_s16(source: buffer, offset: number): number
	local result = buffer_read_i16(source, offset)

	result = bit32_or(result, 0)

	return result
end

-- SECTION load_i32_from_u16
-- NEEDS buffer_read_u16
local rt_load_i32_from_u16 = buffer_read_u16

-- SECTION load_i32
-- NEEDS buffer_read_u32
local rt_load_i32 = buffer_read_u32

-- SECTION load_i64_from_s8
-- NEEDS buffer_read_i8
-- NEEDS into_bits_i64
local function rt_load_i64_from_s8(source: buffer, offset: number): number
	local source = buffer_read_i8(source, offset)

	return if source >= 0 then into_bits_i64(source, 0) else into_bits_i64(source + 0x1_0000_0000, 0xFFFF_FFFF)
end

-- SECTION load_i64_from_u8
-- NEEDS buffer_read_u8
-- NEEDS into_bits_i64
local function rt_load_i64_from_u8(source: buffer, offset: number): number
	local source = buffer_read_u8(source, offset)

	return into_bits_i64(source, 0)
end

-- SECTION load_i64_from_s16
-- NEEDS buffer_read_i16
-- NEEDS into_bits_i64
local function rt_load_i64_from_s16(source: buffer, offset: number): number
	local source = buffer_read_i16(source, offset)

	return if source >= 0 then into_bits_i64(source, 0) else into_bits_i64(source + 0x1_0000_0000, 0xFFFF_FFFF)
end

-- SECTION load_i64_from_u16
-- NEEDS buffer_read_u16
-- NEEDS into_bits_i64
local function rt_load_i64_from_u16(source: buffer, offset: number): number
	local source = buffer_read_u16(source, offset)

	return into_bits_i64(source, 0)
end

-- SECTION load_i64_from_s32
-- NEEDS buffer_read_i32
-- NEEDS into_bits_i64
local function rt_load_i64_from_s32(source: buffer, offset: number): number
	local source = buffer_read_i32(source, offset)

	return if source >= 0 then into_bits_i64(source, 0) else into_bits_i64(source + 0x1_0000_0000, 0xFFFF_FFFF)
end

-- SECTION load_i64_from_u32
-- NEEDS buffer_read_u32
-- NEEDS into_bits_i64
local function rt_load_i64_from_u32(source: buffer, offset: number): number
	local source = buffer_read_u32(source, offset)

	return into_bits_i64(source, 0)
end

-- SECTION load_i64
-- NEEDS buffer_read_u32
-- NEEDS into_bits_i64
local function rt_load_i64(source: buffer, offset: number): number
	local source_1 = buffer_read_u32(source, offset)
	local source_2 = buffer_read_u32(source, offset + 4)

	return into_bits_i64(source_1, source_2)
end

-- SECTION load_f32
-- NEEDS buffer_read_u32
local rt_load_f32 = buffer_read_u32

-- SECTION load_f64
-- NEEDS buffer_read_f64
local rt_load_f64 = buffer_read_f64

-- SECTION store_i32_into_i8
-- NEEDS buffer_write_u8
local rt_store_i32_into_i8 = buffer_write_u8

-- SECTION store_i32_into_i16
-- NEEDS buffer_write_u16
local rt_store_i32_into_i16 = buffer_write_u16

-- SECTION store_i32
-- NEEDS buffer_write_u32
local rt_store_i32 = buffer_write_u32

-- SECTION store_i64_into_i8
-- NEEDS buffer_write_u8
-- NEEDS from_bits_i64
local function rt_store_i64_into_i8(destination: buffer, offset: number, source: number)
	local source_1 = from_bits_i64(source)

	buffer_write_u8(destination, offset, source_1)
end

-- SECTION store_i64_into_i16
-- NEEDS buffer_write_u16
-- NEEDS from_bits_i64
local function rt_store_i64_into_i16(destination: buffer, offset: number, source: number)
	local source_1 = from_bits_i64(source)

	buffer_write_u16(destination, offset, source_1)
end

-- SECTION store_i64_into_i32
-- NEEDS buffer_write_u32
-- NEEDS from_bits_i64
local function rt_store_i64_into_i32(destination: buffer, offset: number, source: number)
	local source_1 = from_bits_i64(source)

	buffer_write_u32(destination, offset, source_1)
end

-- SECTION store_i64
-- NEEDS buffer_write_u32
-- NEEDS from_bits_i64
local function rt_store_i64(destination: buffer, offset: number, source: number)
	local source_1, source_2 = from_bits_i64(source)

	buffer_write_u32(destination, offset + 4, source_2)
	buffer_write_u32(destination, offset, source_1)
end

-- SECTION store_f32
-- NEEDS buffer_write_u32
local rt_store_f32 = buffer_write_u32

-- SECTION store_f64
-- NEEDS buffer_write_f64
local rt_store_f64 = buffer_write_f64

-- SECTION memory_fill
local rt_memory_fill = buffer.fill

-- SECTION memory_copy
local rt_memory_copy = buffer.copy
