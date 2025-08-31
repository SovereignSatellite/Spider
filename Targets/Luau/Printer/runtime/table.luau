-- SECTION elements_type
type Elements<T> = {
	[number]: T,
	count: number,
}

-- SECTION table_type
type Table<T> = {
	[number]: T,
	minimum: number,
	maximum: number,
}

-- SECTION table_new
-- NEEDS table_type
local function rt_table_new<T>(source: T, minimum: number, maximum: number): Table<T>
	local result = table.create(minimum, source)

	result.minimum = minimum
	result.maximum = maximum

	return result
end

-- SECTION table_get
-- NEEDS table_type
local function rt_table_get<T>(source: Table<T>, offset: number): T
	assert(offset < source.minimum, "out of bounds table access")

	return source[offset]
end

-- SECTION table_set
-- NEEDS table_type
local function rt_table_set<T>(destination: Table<T>, offset: number, source: T)
	assert(offset < destination.minimum, "out of bounds table access")

	destination[offset] = source
end

-- SECTION table_size
-- NEEDS table_type
local function rt_table_size<T>(source: Table<T>): number
	return source.minimum
end

-- SECTION table_grow
-- NEEDS table_type
local function rt_table_grow<T>(destination: Table<T>, source: T, size: number): number
	local old = destination.minimum
	local new = old + size

	if new > destination.maximum then
		return 0xFFFF_FFFF
	end

	for offset = old, new do
		destination[offset] = source
	end

	destination.minimum = new

	return old
end

-- SECTION table_fill
-- NEEDS table_type
local function rt_table_fill<T>(destination: Table<T>, offset: number, source: T, size: number)
	assert(offset + size <= destination.minimum, "out of bounds table access")

	for offset = offset, offset + size - 1 do
		destination[offset] = source
	end
end

-- SECTION table_copy
-- NEEDS table_type
local function rt_table_copy<T>(destination: Table<T>, offset_1: number, source: Table<T>, offset_2: number, size: number)
	assert(offset_1 + size <= destination.minimum, "out of bounds table access")
	assert(offset_2 + size <= source.minimum, "out of bounds table access")

	table.move(source, offset_2, offset_2 + size - 1, offset_1, destination)
end

-- SECTION table_init
-- NEEDS elements_type
-- NEEDS table_type
local function rt_table_init<T>(
	destination: Table<T>,
	offset_1: number,
	source: Elements<T>,
	offset_2: number,
	size: number
)
	assert(offset_1 + size <= destination.minimum, "out of bounds table access")
	assert(offset_2 + size <= source.count, "out of bounds elements access")

	table.move(source, offset_2 + 1, offset_2 + size, offset_1, destination)
end

-- SECTION elements_drop
-- NEEDS elements_type
local function rt_elements_drop<T>(elements: Elements<T>)
	if table.isfrozen(elements) then
		return
	end

	table.clear(elements)

	elements.count = 0

	table.freeze(elements)
end
