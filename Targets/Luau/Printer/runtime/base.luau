-- SECTION bit_and
local bit_and = bit32.band

-- SECTION bit_or
local bit_or = bit32.bor

-- SECTION bit_xor
local bit_xor = bit32.bxor

-- SECTION bit_lshift
local bit_lshift = bit32.lshift

-- SECTION bit_rshift
local bit_rshift = bit32.rshift

-- SECTION bit_arshift
local bit_arshift = bit32.arshift

-- SECTION bit_lrotate
local bit_lrotate = bit32.lrotate

-- SECTION bit_rrotate
local bit_rrotate = bit32.rrotate

-- SECTION bit_not
local bit_not = bit32.bnot

-- SECTION bit_replace
local bit_replace = bit32.replace

-- SECTION bit_countlz
local bit_countlz = bit32.countlz

-- SECTION bit_countrz
local bit_countrz = bit32.countrz

-- SECTION math_abs
local math_abs = math.abs

-- SECTION math_sqrt
local math_sqrt = math.sqrt

-- SECTION math_ceil
local math_ceil = math.ceil

-- SECTION math_floor
local math_floor = math.floor

-- SECTION math_round
local math_round = math.round

-- SECTION math_min
local math_min = math.min

-- SECTION math_max
local math_max = math.max

-- SECTION math_fmod
local math_fmod = math.fmod

-- SECTION vector_create
local vector_create = vector.create

-- SECTION vector_abs
local vector_abs = vector.abs

-- SECTION vector_min
local vector_min = vector.min

-- SECTION vector_max
local vector_max = vector.max

-- SECTION raw_memory
local RAW_MEMORY = buffer.create(8)

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
-- NEEDS raw_memory
local function is_positive(source: number): boolean
	buffer_write_f64(RAW_MEMORY, 0, source)

	return buffer_read_i8(RAW_MEMORY, 7) >= 0
end
