-- SECTION table_new
require("table.new")

local function rt_table_new(initializer, minimum, maximum)
	local result = table.new(minimum, 3)

	for offset, element in pairs(initializer) do
		result[offset] = element
	end

	result.minimum = minimum
	result.maximum = maximum

	return result
end

-- SECTION table_get
local function rt_table_get(source, offset)
	if offset < 0 or offset >= source.minimum then
		error("out of bounds table get")
	end

	return source[offset]
end

-- SECTION table_set
local function rt_table_set(destination, offset, source)
	if offset < 0 or offset >= destination.minimum then
		error("out of bounds table set")
	end

	destination[offset] = source
end

-- SECTION table_size
local function rt_table_size(source)
	return source.minimum
end

-- SECTION table_grow
-- NEEDS convert_i32_to_u32
local function rt_table_grow(destination, source, size)
	local size = convert_i32_to_u32(size)
	local old = destination.minimum

	if size == 0 then
		return old
	end

	local new = old + size

	if new > destination.maximum then
		return -1
	end

	for offset = old, new do
		destination[offset] = source
	end

	destination.minimum = new

	return old
end

-- SECTION table_fill
local function rt_table_fill(destination, offset, source, size)
	if size < 0 or offset < 0 or offset + size > destination.minimum then
		error("out of bounds table fill")
	end

	for offset = offset, offset + size - 1 do
		destination[offset] = source
	end
end

-- SECTION table_copy
local function rt_table_copy(destination, offset_1, source, offset_2, size)
	if
		size < 0
		or offset_1 < 0
		or offset_2 < 0
		or offset_1 + size > destination.minimum
		or offset_2 + size > source.minimum
	then
		error("out of bounds table copy")
	end

	table.move(source, offset_2, offset_2 + size - 1, offset_1, destination)
end

-- SECTION table_drop
require("table.clear")

local function rt_table_drop(destination)
	table.clear(destination)

	destination.minimum = 0
	destination.maximum = 0
end
