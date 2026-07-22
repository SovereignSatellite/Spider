-- SECTION ffi
local ffi = require("ffi")

-- SECTION ffi_cast
-- NEEDS ffi
local ffi_cast = ffi.cast

-- SECTION i64_type
-- NEEDS ffi
local i64_type = ffi.typeof("int64_t")

-- SECTION cast_i64
-- NEEDS ffi_cast
-- NEEDS i64_type
local function cast_i64(source)
	return ffi_cast(i64_type, source)
end

-- SECTION u64_type
-- NEEDS ffi
local u64_type = ffi.typeof("uint64_t")

-- SECTION cast_u64
-- NEEDS ffi_cast
-- NEEDS u64_type
local function cast_u64(source)
	return ffi_cast(u64_type, source)
end

-- SECTION c_calloc
-- NEEDS ffi
ffi.cdef([[
void *calloc(size_t count, size_t size);
]])

-- SECTION c_free
-- NEEDS ffi
ffi.cdef([[
void free(void *ptr);
]])

-- SECTION any_type
-- NEEDS ffi
ffi.cdef([[
union Any {
    int8_t i8;
    int16_t i16;
    int32_t i32;
    int64_t i64;

    uint8_t u8;
    uint16_t u16;
    uint32_t u32;
    uint64_t u64;

    float f32;
    double f64;
};
]])

-- SECTION u8_pointer_type
-- NEEDS ffi
local u8_pointer_type = ffi.typeof("uint8_t *")

-- SECTION cast_u8_pointer
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local function cast_u8_pointer(source)
	return ffi_cast(u8_pointer_type, source)
end

-- SECTION any_pointer_type
-- NEEDS any_type
-- NEEDS ffi
local any_pointer_type = ffi.typeof("union Any *")

-- SECTION cast_any_pointer
-- NEEDS any_pointer_type
-- NEEDS ffi_cast
local function cast_any_pointer(source)
	return ffi_cast(any_pointer_type, source)
end

-- SECTION memory_type
-- NEEDS any_type
-- NEEDS ffi
ffi.cdef([[
struct Memory {
    union Any *data;
    uint32_t size;
};
]])

local memory_type = ffi.typeof("struct Memory")

-- SECTION transmute_n32
-- NEEDS ffi
ffi.cdef([[
union transmute_n32 {
    int32_t i32;
    float f32;
};
]])

local TRANSMUTE_N32 = ffi.new("union transmute_n32")

-- SECTION transmute_n64
-- NEEDS ffi
ffi.cdef([[
union transmute_n64 {
    int64_t i64;
    double f64;
};
]])

local TRANSMUTE_N64 = ffi.new("union transmute_n64")

-- SECTION from_bits_f32
-- NEEDS transmute_n32
local function from_bits_f32(source)
	-- LuaJIT reserves NaN payloads for tagged values, so transmuted NaNs use a
	-- canonical payload.
	if source > 0x7F800000 then
		source = 0x7FC00000
	elseif source < 0 and source > -0x00800000 then
		source = -0x00400000
	end

	TRANSMUTE_N32.i32 = source

	return TRANSMUTE_N32.f32
end

-- SECTION into_bits_f32
-- NEEDS transmute_n32
local function into_bits_f32(source)
	TRANSMUTE_N32.f32 = source

	return TRANSMUTE_N32.i32
end

-- SECTION from_bits_f64
-- NEEDS transmute_n64
local function from_bits_f64(source)
	-- LuaJIT reserves NaN payloads for tagged values, so transmuted NaNs use a
	-- canonical payload.
	if source > 0x7FF0000000000000LL then
		source = 0x7FF8000000000000LL
	elseif source < 0LL and source > 0xFFF0000000000000LL then
		source = 0xFFF8000000000000LL
	end

	TRANSMUTE_N64.i64 = source

	return TRANSMUTE_N64.f64
end

-- SECTION into_bits_f64
-- NEEDS transmute_n64
local function into_bits_f64(source)
	TRANSMUTE_N64.f64 = source

	return TRANSMUTE_N64.i64
end
