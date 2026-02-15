-- SECTION integer_create
local integer_create = integer.create

-- SECTION integer_tonumber
local integer_tonumber = integer.tonumber

-- SECTION integer_and
local integer_and = integer.band

-- SECTION integer_sub
local integer_sub = integer.sub

-- SECTION integer_mul
local integer_mul = integer.mul

-- SECTION integer_lshift
local integer_lshift = integer.lshift

-- SECTION integer_rshift
local integer_rshift = integer.rshift

-- SECTION integer_arshift
local integer_arshift = integer.arshift

-- SECTION integer_lt
local integer_lt = integer.lt

-- SECTION integer_ult
local integer_ult = integer.ult

-- SECTION integer_le
local integer_le = integer.le

-- SECTION integer_ule
local integer_ule = integer.ule

-- SECTION convert_f64_to_u64
-- NEEDS bit32_or
-- NEEDS from_halves_i64
local function convert_f64_to_u64(source: number): integer
	local source_1 = bit32_or(source % 0x1_0000_0000, 0)
	local source_2 = bit32_or(source / 0x1_0000_0000, 0)

	source = from_halves_i64(source_1, source_2)

	return source
end
