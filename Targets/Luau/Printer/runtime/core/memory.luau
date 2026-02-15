-- SECTION memory_type
type Memory = {
	[number]: buffer,
	maximum: number,
}

-- SECTION memory_new
-- NEEDS memory_type
local function rt_memory_new(initializer: { string }, minimum: number, maximum: number): Memory
	local data = buffer.create(minimum)

	for offset, content in initializer do
		buffer.writestring(data, offset, content)
	end

	return { data, maximum = maximum }
end

-- SECTION load_i32_from_s8
-- NEEDS bit32_or
-- NEEDS buffer_read_i8
-- NEEDS memory_type
local function rt_load_i32_from_s8(source: Memory, offset: number): number
	local result = buffer_read_i8(source[1], offset)

	result = bit32_or(result, 0)

	return result
end

-- SECTION load_i32_from_u8
-- NEEDS buffer_read_u8
-- NEEDS memory_type
local function rt_load_i32_from_u8(source: Memory, offset: number): number
	local result = buffer_read_u8(source[1], offset)

	return result
end

-- SECTION load_i32_from_s16
-- NEEDS bit32_or
-- NEEDS buffer_read_i16
-- NEEDS memory_type
local function rt_load_i32_from_s16(source: Memory, offset: number): number
	local result = buffer_read_i16(source[1], offset)

	result = bit32_or(result, 0)

	return result
end

-- SECTION load_i32_from_u16
-- NEEDS buffer_read_u16
-- NEEDS memory_type
local function rt_load_i32_from_u16(source: Memory, offset: number): number
	local result = buffer_read_u16(source[1], offset)

	return result
end

-- SECTION load_i32
-- NEEDS buffer_read_u32
-- NEEDS memory_type
local function rt_load_i32(source: Memory, offset: number): number
	local result = buffer_read_u32(source[1], offset)

	return result
end

-- SECTION load_i64_from_s8
-- NEEDS buffer_read_i8
-- NEEDS integer_create
-- NEEDS memory_type
local function rt_load_i64_from_s8(source: Memory, offset: number): integer
	local result = buffer_read_i8(source[1], offset)

	result = integer_create(result)

	return result
end

-- SECTION load_i64_from_u8
-- NEEDS buffer_read_u8
-- NEEDS integer_create
-- NEEDS memory_type
local function rt_load_i64_from_u8(source: Memory, offset: number): integer
	local result = buffer_read_u8(source[1], offset)

	result = integer_create(result)

	return result
end

-- SECTION load_i64_from_s16
-- NEEDS buffer_read_i16
-- NEEDS integer_create
-- NEEDS memory_type
local function rt_load_i64_from_s16(source: Memory, offset: number): integer
	local result = buffer_read_i16(source[1], offset)

	result = integer_create(result)

	return result
end

-- SECTION load_i64_from_u16
-- NEEDS buffer_read_u16
-- NEEDS integer_create
-- NEEDS memory_type
local function rt_load_i64_from_u16(source: Memory, offset: number): integer
	local result = buffer_read_u16(source[1], offset)

	result = integer_create(result)

	return result
end

-- SECTION load_i64_from_s32
-- NEEDS buffer_read_i32
-- NEEDS integer_create
-- NEEDS memory_type
local function rt_load_i64_from_s32(source: Memory, offset: number): integer
	local result = buffer_read_i32(source[1], offset)

	result = integer_create(result)

	return result
end

-- SECTION load_i64_from_u32
-- NEEDS buffer_read_u32
-- NEEDS integer_create
-- NEEDS memory_type
local function rt_load_i64_from_u32(source: Memory, offset: number): integer
	local result = buffer_read_u32(source[1], offset)

	result = integer_create(result)

	return result
end

-- SECTION load_i64
-- NEEDS buffer_read_integer
-- NEEDS memory_type
local function rt_load_i64(source: Memory, offset: number): integer
	local result = buffer_read_integer(source[1], offset)

	return result
end

-- SECTION load_f32
-- NEEDS buffer_read_u32
-- NEEDS memory_type
local function rt_load_f32(source: Memory, offset: number): number
	local result = buffer_read_u32(source[1], offset)

	return result
end

-- SECTION load_f64
-- NEEDS buffer_read_f64
-- NEEDS memory_type
local function rt_load_f64(source: Memory, offset: number): number
	local result = buffer_read_f64(source[1], offset)

	return result
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
-- NEEDS into_halves_i64
-- NEEDS memory_type
local function rt_store_i64_into_i8(destination: Memory, offset: number, source: integer)
	source = into_halves_i64(source)

	buffer_write_u8(destination[1], offset, source)
end

-- SECTION store_i64_into_i16
-- NEEDS buffer_write_u16
-- NEEDS into_halves_i64
-- NEEDS memory_type
local function rt_store_i64_into_i16(destination: Memory, offset: number, source: integer)
	source = into_halves_i64(source)

	buffer_write_u16(destination[1], offset, source)
end

-- SECTION store_i64_into_i32
-- NEEDS buffer_write_u32
-- NEEDS into_halves_i64
-- NEEDS memory_type
local function rt_store_i64_into_i32(destination: Memory, offset: number, source: integer)
	source = into_halves_i64(source)

	buffer_write_u32(destination[1], offset, source)
end

-- SECTION store_i64
-- NEEDS buffer_write_integer
-- NEEDS memory_type
local function rt_store_i64(destination: Memory, offset: number, source: integer)
	buffer_write_integer(destination[1], offset, source)
end

-- SECTION store_f32
-- NEEDS buffer_write_f32
-- NEEDS memory_type
local function rt_store_f32(destination: Memory, offset: number, source: number)
	buffer_write_u32(destination[1], offset, source)
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
	return buffer.len(source[1])
end

-- SECTION memory_grow
-- NEEDS memory_type
local function rt_memory_grow(destination: Memory, size: number): number
	local old = buffer.len(destination[1])
	local new = old + size

	if new > destination.maximum then
		return 0xFFFF_FFFF
	end

	local ok, result = pcall(buffer.create, new)

	if not ok then
		return 0xFFFF_FFFF
	end

	buffer.copy(result, 0, destination[1])

	destination[1] = result

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

-- SECTION memory_drop
-- NEEDS memory_type
local function rt_memory_drop(destination: Memory)
	if table.isfrozen(destination) then
		return
	end

	table.clear(destination)

	destination[1] = buffer.create(0)

	destination.maximum = 0

	table.freeze(destination)
end
