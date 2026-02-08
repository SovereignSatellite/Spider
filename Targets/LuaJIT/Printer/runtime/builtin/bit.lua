-- SECTION bit
local bit = require("bit")

-- SECTION force_u32
local function force_u32(source)
	if source >= 0 then
		return source
	else
		return source + 0x100000000
	end
end

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
