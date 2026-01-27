-- SECTION native_f32
-- NEEDS ffi
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local NATIVE_F32 = (function()
	local FUNCTION_ALIGNMENT = 32

	local function load_code_x64()
		return "\x66\x0F\x6E\xC7\xF3\x0F\x51\xC0\x66\x0F\x7E\xC0\xC3\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\x66\x0F\x6E\xC7\x66\x0F\x6E\xCE\xF3\x0F\x58\xC1\x66\x0F\x7E\xC0\xC3\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\x66\x0F\x6E\xC7\x66\x0F\x6E\xCE\xF3\x0F\x5C\xC1\x66\x0F\x7E\xC0\xC3\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\x66\x0F\x6E\xC7\x66\x0F\x6E\xCE\xF3\x0F\x59\xC1\x66\x0F\x7E\xC0\xC3\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\x66\x0F\x6E\xC7\x66\x0F\x6E\xCE\xF3\x0F\x5E\xC1\x66\x0F\x7E\xC0\xC3\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC"
	end

	local function load_code_invalid()
		return string.rep("\xFF", 5 * FUNCTION_ALIGNMENT)
	end

	local function load_memory_mapped(code)
		ffi.cdef([[
            void* mmap(void *addr, size_t length, int prot, int flags, int fd, size_t offset);
            int mprotect(void *addr, size_t len, int prot);
            int munmap(void *addr, size_t length);
        ]])

		local PROT_READ = 0x01
		local PROT_WRITE = 0x02
		local MAP_PRIVATE = 0x02
		local MAP_ANONYMOUS = 0x20

		local page = ffi_cast(
			u8_pointer_type,
			ffi.C.mmap(nil, #code, PROT_READ + PROT_WRITE, MAP_PRIVATE + MAP_ANONYMOUS, -1, 0)
		)

		if page == ffi_cast(u8_pointer_type, -1) then
			error("failed to allocate code page for `f32`")
		end

		ffi.copy(page, code, #code)

		local PROT_EXEC = 0x04

		if ffi.C.mprotect(page, #code, PROT_READ + PROT_EXEC) ~= 0 then
			error("failed to set permissions of code page for `f32`")
		end

		return page
	end

	local function load_memory_invalid(code)
		ffi.cdef([[
		    void *malloc(size_t size);
        ]])

		local page = ffi.C.malloc(#code)

		if page == nil then
			error("failed to allocate memory for `f32`")
		end

		ffi.copy(page, code, #code)

		return ffi_cast(u8_pointer_type, page)
	end

	local code

	if jit.arch == "x64" then
		code = load_code_x64()
	else
		code = load_code_invalid()
	end

	local memory

	if jit.os == "Linux" or jit.os == "OSX" then
		memory = load_memory_mapped(code)
	else
		memory = load_memory_invalid(code)
	end

	local f32_f32_to_f32 = ffi.typeof("int32_t (*)(int32_t, int32_t)")
	local f32_to_f32 = ffi.typeof("int32_t (*)(int32_t)")

	return {
		square_root_f32 = ffi_cast(f32_to_f32, memory + 0 * FUNCTION_ALIGNMENT),
		add_f32 = ffi_cast(f32_f32_to_f32, memory + 1 * FUNCTION_ALIGNMENT),
		subtract_f32 = ffi_cast(f32_f32_to_f32, memory + 2 * FUNCTION_ALIGNMENT),
		multiply_f32 = ffi_cast(f32_f32_to_f32, memory + 3 * FUNCTION_ALIGNMENT),
		divide_f32 = ffi_cast(f32_f32_to_f32, memory + 4 * FUNCTION_ALIGNMENT),
	}
end)()

-- SECTION from_bits_f32
-- NEEDS transmute_n32
local function from_bits_f32(source)
	-- LuaJIT does NaN tagging. This means we must manually make sure we don't
	-- accidentally create a broken value when transmuting.
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

-- SECTION absolute_f32
-- NEEDS bit_and
local function rt_absolute_f32(source)
	source = bit_and(source, 0x7FFFFFFF)

	return source
end

-- SECTION negate_f32
-- NEEDS bit_xor
local function rt_negate_f32(source)
	source = bit_xor(source, 0x80000000)

	return source
end

-- SECTION round_up_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_ceil
local function rt_round_up_f32(source)
	source = from_bits_f32(source)
	source = math_ceil(source)
	source = into_bits_f32(source)

	return source
end

-- SECTION round_down_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_floor
local function rt_round_down_f32(source)
	source = from_bits_f32(source)
	source = math_floor(source)
	source = into_bits_f32(source)

	return source
end

-- SECTION truncate_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_modf
local function rt_truncate_f32(source)
	source = from_bits_f32(source)
	source = math_modf(source)
	source = into_bits_f32(source)

	return source
end

-- SECTION nearest_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_abs
-- NEEDS math_modf
local function rt_nearest_f32(source)
	local native = from_bits_f32(source)

	native = math_abs(native)

	local rounded, remainder = math_modf(native)

	if remainder > 0.5 or (remainder == 0.5 and rounded % 2 == 1) then
		rounded = rounded + 1
	end

	if source < 0 then
		rounded = -rounded
	end

	source = into_bits_f32(rounded)

	return source
end

-- SECTION square_root_f32
-- NEEDS native_f32
local rt_square_root_f32 = NATIVE_F32.square_root_f32

-- SECTION add_f32
-- NEEDS native_f32
local rt_add_f32 = NATIVE_F32.add_f32

-- SECTION subtract_f32
-- NEEDS native_f32
local rt_subtract_f32 = NATIVE_F32.subtract_f32

-- SECTION multiply_f32
-- NEEDS native_f32
local rt_multiply_f32 = NATIVE_F32.multiply_f32

-- SECTION divide_f32
-- NEEDS native_f32
local rt_divide_f32 = NATIVE_F32.divide_f32

-- SECTION minimum_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_min
local function rt_minimum_f32(lhs, rhs)
	if rhs >= 0 then
		lhs, rhs = rhs, lhs
	end

	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	local result = math_min(lhs, rhs)

	result = into_bits_f32(result)

	return result
end

-- SECTION maximum_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f32
-- NEEDS math_max
local function rt_maximum_f32(lhs, rhs)
	if rhs < 0 then
		lhs, rhs = rhs, lhs
	end

	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	local result = math_max(lhs, rhs)

	result = into_bits_f32(result)

	return result
end

-- SECTION copy_sign_f32
-- NEEDS bit_and
-- NEEDS bit_or
local function rt_copy_sign_f32(lhs, rhs)
	lhs = bit_and(lhs, 0x7FFFFFFF)
	rhs = bit_and(rhs, 0x80000000)

	local result = bit_or(lhs, rhs)

	return result
end

-- SECTION equal_f32
-- NEEDS from_bits_f32
local function rt_equal_f32(lhs, rhs)
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs == rhs
end

-- SECTION not_equal_f32
-- NEEDS from_bits_f32
local function rt_not_equal_f32(lhs, rhs)
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs ~= rhs
end

-- SECTION less_than_f32
-- NEEDS from_bits_f32
local function rt_less_than_f32(lhs, rhs)
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs < rhs
end

-- SECTION greater_than_f32
-- NEEDS from_bits_f32
local function rt_greater_than_f32(lhs, rhs)
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs > rhs
end

-- SECTION less_than_equal_f32
-- NEEDS from_bits_f32
local function rt_less_than_equal_f32(lhs, rhs)
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs <= rhs
end

-- SECTION greater_than_equal_f32
-- NEEDS from_bits_f32
local function rt_greater_than_equal_f32(lhs, rhs)
	lhs = from_bits_f32(lhs)
	rhs = from_bits_f32(rhs)

	return lhs >= rhs
end

-- SECTION widen_f32
-- NEEDS from_bits_f32
-- NEEDS into_bits_f64
local function rt_widen_f32(source)
	source = from_bits_f32(source)
	source = into_bits_f64(source)

	return source
end

-- SECTION saturate_f32_to_s32
-- NEEDS force_i32
-- NEEDS from_bits_f32
-- NEEDS math_modf
local function rt_saturate_f32_to_s32(source)
	source = from_bits_f32(source)

	if source >= 0x80000000 then
		source = force_i32(0x7FFFFFFF)
	elseif source <= -0x80000000 then
		source = force_i32(-0x80000000)
	elseif source ~= source then
		source = force_i32(0)
	else
		source = math_modf(source)
		source = force_i32(source)
	end

	return source
end

-- SECTION truncate_f32_to_s32
-- NEEDS force_i32
-- NEEDS from_bits_f32
-- NEEDS math_modf
local function rt_truncate_f32_to_s32(source)
	source = from_bits_f32(source)
	source = math_modf(source)

	if source >= 0x80000000 or source < -0x80000000 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = force_i32(source)

	return source
end

-- SECTION saturate_f32_to_u32
-- NEEDS force_i32
-- NEEDS from_bits_f32
-- NEEDS math_floor
local function rt_saturate_f32_to_u32(source)
	source = from_bits_f32(source)

	if source >= 0x100000000 then
		source = force_i32(0xFFFFFFFF)
	elseif source <= 0 or source ~= source then
		source = force_i32(0)
	else
		source = math_floor(source)
		source = force_i32(source)
	end

	return source
end

-- SECTION truncate_f32_to_u32
-- NEEDS force_i32
-- NEEDS from_bits_f32
-- NEEDS math_modf
local function rt_truncate_f32_to_u32(source)
	source = from_bits_f32(source)
	source = math_modf(source)

	if source >= 0x100000000 or source < 0 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = force_i32(source)

	return source
end

-- SECTION saturate_f32_to_s64
-- NEEDS ffi_cast
-- NEEDS from_bits_f32
-- NEEDS i64_type
-- NEEDS math_modf
local function rt_saturate_f32_to_s64(source)
	source = from_bits_f32(source)

	if source >= 0x8000000000000000 then
		source = 0x7FFFFFFFFFFFFFFFLL
	elseif source <= -0x8000000000000000 then
		source = -0x8000000000000000LL
	elseif source ~= source then
		source = 0LL
	else
		source = math_modf(source)
		source = ffi_cast(i64_type, source)
	end

	return source
end

-- SECTION truncate_f32_to_s64
-- NEEDS ffi_cast
-- NEEDS from_bits_f32
-- NEEDS i64_type
-- NEEDS math_modf
local function rt_truncate_f32_to_s64(source)
	source = from_bits_f32(source)
	source = math_modf(source)

	if source >= 0x8000000000000000 or source < -0x8000000000000000 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = ffi_cast(i64_type, source)

	return source
end

-- SECTION saturate_f32_to_u64
-- NEEDS ffi_cast
-- NEEDS from_bits_f32
-- NEEDS i64_type
-- NEEDS math_floor
-- NEEDS u64_type
local function rt_saturate_f32_to_u64(source)
	source = from_bits_f32(source)

	if source >= 0x10000000000000000 then
		source = 0xFFFFFFFFFFFFFFFFLL
	elseif source <= 0 or source ~= source then
		source = 0LL
	else
		source = math_floor(source)
		source = ffi_cast(u64_type, source)
		source = ffi_cast(i64_type, source)
	end

	return source
end

-- SECTION truncate_f32_to_u64
-- NEEDS ffi_cast
-- NEEDS from_bits_f32
-- NEEDS i64_type
-- NEEDS math_modf
-- NEEDS u64_type
local function rt_truncate_f32_to_u64(source)
	source = from_bits_f32(source)
	source = math_modf(source)

	if source >= 0x10000000000000000 or source < 0 then
		error("integer overflow")
	elseif source ~= source then
		error("invalid conversion to integer")
	end

	source = ffi_cast(u64_type, source)
	source = ffi_cast(i64_type, source)

	return source
end

-- SECTION transmute_f32_to_i32
local function rt_transmute_f32_to_i32(source)
	return source
end
