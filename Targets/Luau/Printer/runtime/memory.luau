-- SECTION memory_type
type Memory = {
	[number]: buffer,
	maximum: number,
}

-- SECTION data_type
type Data = {
	[number]: buffer,
}

-- SECTION memory_new
-- NEEDS memory_type
local function rt_memory_new(minimum: number, maximum: number): Memory
	return {
		buffer.create(minimum * 0x1_0000),
		maximum = maximum,
	}
end

-- SECTION load_i32_from_s8
-- NEEDS bit_or
-- NEEDS buffer_read_i8
-- NEEDS memory_type
local function rt_load_i32_from_s8(source: Memory, offset: number): number
	return bit_or(buffer_read_i8(source[1], offset), 0)
end

-- SECTION load_i32_from_u8
-- NEEDS buffer_read_u8
-- NEEDS memory_type
local function rt_load_i32_from_u8(source: Memory, offset: number): number
	return buffer_read_u8(source[1], offset)
end

-- SECTION load_i32_from_s16
-- NEEDS bit_or
-- NEEDS buffer_read_i16
-- NEEDS memory_type
local function rt_load_i32_from_s16(source: Memory, offset: number): number
	return bit_or(buffer_read_i16(source[1], offset), 0)
end

-- SECTION load_i32_from_u16
-- NEEDS buffer_read_u16
-- NEEDS memory_type
local function rt_load_i32_from_u16(source: Memory, offset: number): number
	return buffer_read_u16(source[1], offset)
end

-- SECTION load_i32
-- NEEDS buffer_read_u32
-- NEEDS memory_type
local function rt_load_i32(source: Memory, offset: number): number
	return buffer_read_u32(source[1], offset)
end

-- SECTION load_i64_from_s8
-- NEEDS buffer_read_i8
-- NEEDS create_i64_from_u32
-- NEEDS memory_type
local function rt_load_i64_from_s8(source: Memory, offset: number): number
	local source = buffer_read_i8(source[1], offset)

	return if source >= 0
		then rt_create_i64_from_u32(source, 0)
		else rt_create_i64_from_u32(source + 0x1_0000_0000, 0xFFFF_FFFF)
end

-- SECTION load_i64_from_u8
-- NEEDS buffer_read_u8
-- NEEDS create_i64_from_u32
-- NEEDS memory_type
local function rt_load_i64_from_u8(source: Memory, offset: number): number
	local source = buffer_read_u8(source[1], offset)

	return rt_create_i64_from_u32(source, 0)
end

-- SECTION load_i64_from_s16
-- NEEDS buffer_read_i16
-- NEEDS create_i64_from_u32
-- NEEDS memory_type
local function rt_load_i64_from_s16(source: Memory, offset: number): number
	local source = buffer_read_i16(source[1], offset)

	return if source >= 0
		then rt_create_i64_from_u32(source, 0)
		else rt_create_i64_from_u32(source + 0x1_0000_0000, 0xFFFF_FFFF)
end

-- SECTION load_i64_from_u16
-- NEEDS buffer_read_u16
-- NEEDS create_i64_from_u32
-- NEEDS memory_type
local function rt_load_i64_from_u16(source: Memory, offset: number): number
	local source = buffer_read_u16(source[1], offset)

	return rt_create_i64_from_u32(source, 0)
end

-- SECTION load_i64_from_s32
-- NEEDS buffer_read_i32
-- NEEDS create_i64_from_u32
-- NEEDS memory_type
local function rt_load_i64_from_s32(source: Memory, offset: number): number
	local source = buffer_read_i32(source[1], offset)

	return if source >= 0
		then rt_create_i64_from_u32(source, 0)
		else rt_create_i64_from_u32(source + 0x1_0000_0000, 0xFFFF_FFFF)
end

-- SECTION load_i64_from_u32
-- NEEDS buffer_read_u32
-- NEEDS create_i64_from_u32
-- NEEDS memory_type
local function rt_load_i64_from_u32(source: Memory, offset: number): number
	local source = buffer_read_u32(source[1], offset)

	return rt_create_i64_from_u32(source, 0)
end

-- SECTION load_i64
-- NEEDS buffer_read_u32
-- NEEDS create_i64_from_u32
-- NEEDS memory_type
local function rt_load_i64(source: Memory, offset: number): number
	local memory = source[1]
	local source_1 = buffer_read_u32(memory, offset)
	local source_2 = buffer_read_u32(memory, offset + 4)

	return rt_create_i64_from_u32(source_1, source_2)
end

-- SECTION load_f32
-- NEEDS buffer_read_f32
-- NEEDS memory_type
-- NEEDS vector_create
local function rt_load_f32(source: Memory, offset: number): vector
	local source = buffer_read_f32(source[1], offset)

	return vector_create(source, 0, 0)
end

-- SECTION load_f64
-- NEEDS buffer_read_f64
-- NEEDS memory_type
local function rt_load_f64(source: Memory, offset: number): number
	return buffer_read_f64(source[1], offset)
end

-- SECTION store_i32_into_i8
-- NEEDS buffer_write_u8
-- NEEDS memory_type
local function rt_store_i32_into_i8(destination: Memory, offset: number, source: number)
	buffer_write_u8(destination[1], offset, source)
end

-- SECTION store_i32_into_i16
-- NEEDS buffer_write_u16
-- NEEDS memory_type
local function rt_store_i32_into_i16(destination: Memory, offset: number, source: number)
	buffer_write_u16(destination[1], offset, source)
end

-- SECTION store_i32
-- NEEDS buffer_write_u32
-- NEEDS memory_type
local function rt_store_i32(destination: Memory, offset: number, source: number)
	buffer_write_u32(destination[1], offset, source)
end

-- SECTION store_i64_into_i8
-- NEEDS buffer_write_u8
-- NEEDS create_u32_from_i64
-- NEEDS memory_type
local function rt_store_i64_into_i8(destination: Memory, offset: number, source: number)
	local source_1, _ = rt_create_u32_from_i64(source)

	buffer_write_u8(destination[1], offset, source_1)
end

-- SECTION store_i64_into_i16
-- NEEDS buffer_write_u16
-- NEEDS create_u32_from_i64
-- NEEDS memory_type
local function rt_store_i64_into_i16(destination: Memory, offset: number, source: number)
	local source_1, _ = rt_create_u32_from_i64(source)

	buffer_write_u16(destination[1], offset, source_1)
end

-- SECTION store_i64_into_i32
-- NEEDS buffer_write_u32
-- NEEDS create_u32_from_i64
-- NEEDS memory_type
local function rt_store_i64_into_i32(destination: Memory, offset: number, source: number)
	local source_1, _ = rt_create_u32_from_i64(source)

	buffer_write_u32(destination[1], offset, source_1)
end

-- SECTION store_i64
-- NEEDS buffer_write_u32
-- NEEDS create_u32_from_i64
-- NEEDS memory_type
local function rt_store_i64(destination: Memory, offset: number, source: number)
	local source_1, source_2 = rt_create_u32_from_i64(source)

	buffer_write_u32(destination[1], offset + 4, source_2)
	buffer_write_u32(destination[1], offset, source_1)
end

-- SECTION store_f32
-- NEEDS buffer_write_f32
-- NEEDS memory_type
local function rt_store_f32(destination: Memory, offset: number, source: vector)
	buffer_write_f32(destination[1], offset, source.x)
end

-- SECTION store_f64
-- NEEDS buffer_write_f64
-- NEEDS memory_type
local function rt_store_f64(destination: Memory, offset: number, source: number)
	buffer_write_f64(destination[1], offset, source)
end

-- SECTION memory_size
-- NEEDS memory_type
local function rt_memory_size(source: Memory): number
	return buffer.len(source[1]) // 0x1_0000
end

-- SECTION memory_grow
-- NEEDS memory_type
local function rt_memory_grow(destination: Memory, size: number): number
	local old = buffer.len(destination[1]) // 0x1_0000
	local new = old + size

	if new > destination.maximum then
		return 0xFFFF_FFFF
	end

	local ok, result = pcall(buffer.create, new * 0x1_0000)

	if not ok then
		return 0xFFFF_FFFF
	end

	buffer.copy(result, 0, destination[1])

	destination[1] = result
	destination.minimum = new

	return old
end

-- SECTION memory_fill
-- NEEDS memory_type
local function rt_memory_fill(destination: Memory, offset: number, source: number, size: number)
	buffer.fill(destination[1], offset, source, size)
end

-- SECTION memory_copy
-- NEEDS memory_type
local function rt_memory_copy(destination: Memory, offset_1: number, source: Memory, offset_2: number, size: number)
	buffer.copy(destination[1], offset_1, source[1], offset_2, size)
end

-- SECTION memory_init
-- NEEDS data_type
-- NEEDS memory_type
local function rt_memory_init(destination: Memory, offset_1: number, source: Data, offset_2: number, size: number)
	buffer.copy(destination[1], offset_1, source[1], offset_2, size)
end

-- SECTION data_drop
-- NEEDS data_type
local function rt_data_drop(data: Data)
	if table.isfrozen(data) then
		return
	end

	table.clear(data)

	data[1] = buffer.create(0)

	table.freeze(data)
end
