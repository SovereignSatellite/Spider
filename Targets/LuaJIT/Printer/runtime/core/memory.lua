-- SECTION memory_new
-- NEEDS ffi
-- NEEDS ffi_cast
-- NEEDS memory_drop
-- NEEDS memory_grow
-- NEEDS memory_type
-- NEEDS u8_pointer_type
local function rt_memory_new(initializer, minimum, maximum)
	local result = ffi.gc(memory_type(nil, 0, maximum), rt_memory_drop)

	if rt_memory_grow(result, minimum) == -1 then
		error("failed to allocate memory")
	end

	local address = ffi_cast(u8_pointer_type, result.data)

	for offset, content in pairs(initializer) do
		ffi.copy(address + offset, content, #content)
	end

	return result
end

-- SECTION load_i32_from_s8
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_load_i32_from_s8(source, offset)
	if offset < 0 or offset + 1 > source.minimum then
		error("out of bounds memory load")
	end

	local address = ffi_cast(u8_pointer_type, source.data) + offset

	return ffi_cast(any_pointer_type, address).i8
end

-- SECTION load_i32_from_u8
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_load_i32_from_u8(source, offset)
	if offset < 0 or offset + 1 > source.minimum then
		error("out of bounds memory load")
	end

	local address = ffi_cast(u8_pointer_type, source.data) + offset

	return ffi_cast(any_pointer_type, address).u8
end

-- SECTION load_i32_from_s16
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_load_i32_from_s16(source, offset)
	if offset < 0 or offset + 2 > source.minimum then
		error("out of bounds memory load")
	end

	local address = ffi_cast(u8_pointer_type, source.data) + offset

	return ffi_cast(any_pointer_type, address).i16
end

-- SECTION load_i32_from_u16
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_load_i32_from_u16(source, offset)
	if offset < 0 or offset + 2 > source.minimum then
		error("out of bounds memory load")
	end

	local address = ffi_cast(u8_pointer_type, source.data) + offset

	return ffi_cast(any_pointer_type, address).u16
end

-- SECTION load_i32
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_load_i32(source, offset)
	if offset < 0 or offset + 4 > source.minimum then
		error("out of bounds memory load")
	end

	local address = ffi_cast(u8_pointer_type, source.data) + offset

	return ffi_cast(any_pointer_type, address).i32
end

-- SECTION load_i64_from_s8
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS i64_type
-- NEEDS u8_pointer_type
local function rt_load_i64_from_s8(source, offset)
	if offset < 0 or offset + 1 > source.minimum then
		error("out of bounds memory load")
	end

	local address = ffi_cast(u8_pointer_type, source.data) + offset
	local result = ffi_cast(i64_type, ffi_cast(any_pointer_type, address).i8)

	return result
end

-- SECTION load_i64_from_u8
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS i64_type
-- NEEDS u8_pointer_type
local function rt_load_i64_from_u8(source, offset)
	if offset < 0 or offset + 1 > source.minimum then
		error("out of bounds memory load")
	end

	local address = ffi_cast(u8_pointer_type, source.data) + offset
	local result = ffi_cast(i64_type, ffi_cast(any_pointer_type, address).u8)

	return result
end

-- SECTION load_i64_from_s16
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS i64_type
-- NEEDS u8_pointer_type
local function rt_load_i64_from_s16(source, offset)
	if offset < 0 or offset + 2 > source.minimum then
		error("out of bounds memory load")
	end

	local address = ffi_cast(u8_pointer_type, source.data) + offset
	local result = ffi_cast(i64_type, ffi_cast(any_pointer_type, address).i16)

	return result
end

-- SECTION load_i64_from_u16
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS i64_type
-- NEEDS u8_pointer_type
local function rt_load_i64_from_u16(source, offset)
	if offset < 0 or offset + 2 > source.minimum then
		error("out of bounds memory load")
	end

	local address = ffi_cast(u8_pointer_type, source.data) + offset
	local result = ffi_cast(i64_type, ffi_cast(any_pointer_type, address).u16)

	return result
end

-- SECTION load_i64_from_s32
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS i64_type
-- NEEDS u8_pointer_type
local function rt_load_i64_from_s32(source, offset)
	if offset < 0 or offset + 4 > source.minimum then
		error("out of bounds memory load")
	end

	local address = ffi_cast(u8_pointer_type, source.data) + offset
	local result = ffi_cast(i64_type, ffi_cast(any_pointer_type, address).i32)

	return result
end

-- SECTION load_i64_from_u32
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS i64_type
-- NEEDS u8_pointer_type
local function rt_load_i64_from_u32(source, offset)
	if offset < 0 or offset + 4 > source.minimum then
		error("out of bounds memory load")
	end

	local address = ffi_cast(u8_pointer_type, source.data) + offset
	local result = ffi_cast(i64_type, ffi_cast(any_pointer_type, address).u32)

	return result
end

-- SECTION load_i64
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_load_i64(source, offset)
	if offset < 0 or offset + 8 > source.minimum then
		error("out of bounds memory load")
	end

	local address = ffi_cast(u8_pointer_type, source.data) + offset

	return ffi_cast(any_pointer_type, address).i64
end

-- SECTION load_f32
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_load_f32(source, offset)
	if offset < 0 or offset + 4 > source.minimum then
		error("out of bounds memory load")
	end

	local address = ffi_cast(u8_pointer_type, source.data) + offset

	return ffi_cast(any_pointer_type, address).i32
end

-- SECTION load_f64
-- NEEDS load_i64
-- NEEDS transmute_i64_to_f64
local function rt_load_f64(source, offset)
	if offset < 0 or offset + 8 > source.minimum then
		error("out of bounds memory load")
	end

	local address = ffi_cast(u8_pointer_type, source.data) + offset

	return ffi_cast(any_pointer_type, address).i64
end

-- SECTION store_i32_into_i8
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_store_i32_into_i8(destination, offset, source)
	if offset < 0 or offset + 1 > destination.minimum then
		error("out of bounds memory store")
	end

	local address = ffi_cast(u8_pointer_type, destination.data) + offset

	ffi_cast(any_pointer_type, address).i8 = source
end

-- SECTION store_i32_into_i16
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_store_i32_into_i16(destination, offset, source)
	if offset < 0 or offset + 2 > destination.minimum then
		error("out of bounds memory store")
	end

	local address = ffi_cast(u8_pointer_type, destination.data) + offset

	ffi_cast(any_pointer_type, address).i16 = source
end

-- SECTION store_i32
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_store_i32(destination, offset, source)
	if offset < 0 or offset + 4 > destination.minimum then
		error("out of bounds memory store")
	end

	local address = ffi_cast(u8_pointer_type, destination.data) + offset

	ffi_cast(any_pointer_type, address).i32 = source
end

-- SECTION store_i64_into_i8
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_store_i64_into_i8(destination, offset, source)
	if offset < 0 or offset + 1 > destination.minimum then
		error("out of bounds memory store")
	end

	local address = ffi_cast(u8_pointer_type, destination.data) + offset

	ffi_cast(any_pointer_type, address).i8 = source
end

-- SECTION store_i64_into_i16
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_store_i64_into_i16(destination, offset, source)
	if offset < 0 or offset + 2 > destination.minimum then
		error("out of bounds memory store")
	end

	local address = ffi_cast(u8_pointer_type, destination.data) + offset

	ffi_cast(any_pointer_type, address).i16 = source
end

-- SECTION store_i64_into_i32
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_store_i64_into_i32(destination, offset, source)
	if offset < 0 or offset + 4 > destination.minimum then
		error("out of bounds memory store")
	end

	local address = ffi_cast(u8_pointer_type, destination.data) + offset

	ffi_cast(any_pointer_type, address).i32 = source
end

-- SECTION store_i64
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_store_i64(destination, offset, source)
	if offset < 0 or offset + 8 > destination.minimum then
		error("out of bounds memory store")
	end

	local address = ffi_cast(u8_pointer_type, destination.data) + offset

	ffi_cast(any_pointer_type, address).i64 = source
end

-- SECTION store_f32
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_store_f32(destination, offset, source)
	if offset < 0 or offset + 4 > destination.minimum then
		error("out of bounds memory store")
	end

	local address = ffi_cast(u8_pointer_type, destination.data) + offset

	ffi_cast(any_pointer_type, address).i32 = source
end

-- SECTION store_f64
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_store_f64(destination, offset, source)
	if offset < 0 or offset + 8 > destination.minimum then
		error("out of bounds memory store")
	end

	local address = ffi_cast(u8_pointer_type, destination.data) + offset

	ffi_cast(any_pointer_type, address).i64 = source
end

-- SECTION memory_size
-- NEEDS force_i32
local function rt_memory_size(source)
	source = force_i32(source.minimum)

	return source
end

-- SECTION memory_grow
-- NEEDS c_realloc
-- NEEDS ffi
-- NEEDS ffi_cast
-- NEEDS force_u32
-- NEEDS u8_pointer_type
local function rt_memory_grow(destination, size)
	local size = force_u32(size)
	local old = destination.minimum

	if size == 0 then
		return old
	end

	local new = old + size

	if new > destination.maximum then
		return -1
	end

	local data = ffi.C.realloc(destination.data, new)

	if data == nil then
		return -1
	end

	ffi.fill(ffi_cast(u8_pointer_type, data) + old, size, 0)

	destination.data = data
	destination.minimum = new

	return old
end

-- SECTION memory_fill
-- NEEDS ffi
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_memory_fill(destination, offset, source, size)
	if size < 0 or offset < 0 or offset + size > destination.minimum then
		error("out of bounds memory fill")
	end

	local address = ffi_cast(u8_pointer_type, destination.data) + offset

	ffi.fill(address, size, source)
end

-- SECTION memory_copy
-- NEEDS ffi
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function rt_memory_copy(destination, offset_1, source, offset_2, size)
	if
		size < 0
		or offset_1 < 0
		or offset_2 < 0
		or offset_1 + size > destination.minimum
		or offset_2 + size > source.minimum
	then
		error("out of bounds memory copy")
	end

	local address_1 = ffi_cast(u8_pointer_type, destination.data) + offset_1
	local address_2 = ffi_cast(u8_pointer_type, source.data) + offset_2

	ffi.copy(address_1, address_2, size)
end

-- SECTION memory_drop
-- NEEDS c_free
-- NEEDS ffi
local function rt_memory_drop(destination)
	local address = destination.data

	destination.data = nil
	destination.minimum = 0
	destination.maximum = 0

	ffi.C.free(address)
end
