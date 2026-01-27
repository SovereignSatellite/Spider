-- SECTION ffi
local ffi = require("ffi")

-- SECTION c_realloc
-- NEEDS ffi
ffi.cdef([[
void *realloc(void *ptr, size_t size);
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

-- SECTION bit
local bit = require("bit")

-- SECTION force_i32
-- NEEDS bit
local force_i32 = bit.tobit

-- SECTION bit_and
-- NEEDS bit
local bit_and = bit.band

-- SECTION bit_or
-- NEEDS bit
local bit_or = bit.bor

-- SECTION bit_xor
-- NEEDS bit
local bit_xor = bit.bxor

-- SECTION bit_lshift
-- NEEDS bit
local bit_lshift = bit.lshift

-- SECTION bit_rshift
-- NEEDS bit
local bit_rshift = bit.rshift

-- SECTION bit_arshift
-- NEEDS bit
local bit_arshift = bit.arshift

-- SECTION bit_lrotate
-- NEEDS bit
local bit_lrotate = bit.rol

-- SECTION bit_rrotate
-- NEEDS bit
local bit_rrotate = bit.ror

-- SECTION math_abs
local math_abs = math.abs

-- SECTION math_sqrt
local math_sqrt = math.sqrt

-- SECTION math_ceil
local math_ceil = math.ceil

-- SECTION math_floor
local math_floor = math.floor

-- SECTION math_min
local math_min = math.min

-- SECTION math_max
local math_max = math.max

-- SECTION math_fmod
local math_fmod = math.fmod

-- SECTION math_modf
local math_modf = math.modf

-- SECTION convert_i32_to_u32
local function convert_i32_to_u32(source)
	if source < 0 then
		source = source + 0x100000000
	end

	return source
end
