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
	assert(offset >= 0, "out of bounds table access")
	assert(offset < source.minimum, "out of bounds table access")

	return source[offset]
end

-- SECTION table_set
local function rt_table_set(destination, offset, source)
	assert(offset >= 0, "out of bounds table access")
	assert(offset < destination.minimum, "out of bounds table access")

	destination[offset] = source
end

-- SECTION table_size
-- NEEDS force_i32
local function rt_table_size(source)
	return force_i32(source.minimum)
end

-- SECTION table_grow
-- NEEDS force_i32
-- NEEDS force_u32
local function rt_table_grow(destination, source, size)
	local size = force_u32(size)
	local old = destination.minimum

	local new = old + size

	if new > destination.maximum then
		return force_i32(-1)
	end

	for offset = old, new - 1 do
		destination[offset] = source
	end

	destination.minimum = new

	return force_i32(old)
end

-- SECTION table_fill
local function rt_table_fill(destination, offset, source, size)
	assert(size >= 0, "out of bounds table access")
	assert(offset >= 0, "out of bounds table access")
	assert(offset + size <= destination.minimum, "out of bounds table access")

	for offset = offset, offset + size - 1 do
		destination[offset] = source
	end
end

-- SECTION table_copy
local function rt_table_copy(destination, offset_1, source, offset_2, size)
	assert(size >= 0, "out of bounds table access")
	assert(offset_1 >= 0, "out of bounds table access")
	assert(offset_1 + size <= destination.minimum, "out of bounds table access")
	assert(offset_2 >= 0, "out of bounds table access")
	assert(offset_2 + size <= source.minimum, "out of bounds table access")

	table.move(source, offset_2, offset_2 + size - 1, offset_1, destination)
end

-- SECTION table_drop
require("table.clear")

local function rt_table_drop(destination)
	table.clear(destination)

	destination.minimum = 0
	destination.maximum = 0
end
