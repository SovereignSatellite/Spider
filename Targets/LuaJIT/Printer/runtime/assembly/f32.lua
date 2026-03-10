-- SECTION native_f32
-- NEEDS ffi
-- NEEDS ffi_cast
-- NEEDS u8_pointer_type
local NATIVE_F32 = (function()
	local FUNCTION_ALIGNMENT = 32

	local function load_code_x64()
		return "\x66\x0F\x6E\xC7\xF3\x0F\x51\xC0\x66\x0F\x7E\xC0\xC3\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\x66\x0F\x6E\xC7\x66\x0F\x6E\xCE\xF3\x0F\x58\xC1\x66\x0F\x7E\xC0\xC3\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\x66\x0F\x6E\xC7\x66\x0F\x6E\xCE\xF3\x0F\x5C\xC1\x66\x0F\x7E\xC0\xC3\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\x66\x0F\x6E\xC7\x66\x0F\x6E\xCE\xF3\x0F\x59\xC1\x66\x0F\x7E\xC0\xC3\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\x66\x0F\x6E\xC7\x66\x0F\x6E\xCE\xF3\x0F\x5E\xC1\x66\x0F\x7E\xC0\xC3\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC"
	end

	local function load_code_arm64()
		return "\x00\x00\x27\x1E\x00\xC0\x21\x1E\x00\x00\x26\x1E\xC0\x03\x5F\xD6\x1F\x20\x03\xD5\x1F\x20\x03\xD5\x1F\x20\x03\xD5\x1F\x20\x03\xD5\x00\x00\x27\x1E\x21\x00\x27\x1E\x00\x28\x21\x1E\x00\x00\x26\x1E\xC0\x03\x5F\xD6\x1F\x20\x03\xD5\x1F\x20\x03\xD5\x1F\x20\x03\xD5\x00\x00\x27\x1E\x21\x00\x27\x1E\x00\x38\x21\x1E\x00\x00\x26\x1E\xC0\x03\x5F\xD6\x1F\x20\x03\xD5\x1F\x20\x03\xD5\x1F\x20\x03\xD5\x00\x00\x27\x1E\x21\x00\x27\x1E\x00\x08\x21\x1E\x00\x00\x26\x1E\xC0\x03\x5F\xD6\x1F\x20\x03\xD5\x1F\x20\x03\xD5\x1F\x20\x03\xD5\x00\x00\x27\x1E\x21\x00\x27\x1E\x00\x18\x21\x1E\x00\x00\x26\x1E\xC0\x03\x5F\xD6\x1F\x20\x03\xD5\x1F\x20\x03\xD5\x1F\x20\x03\xD5"
	end

	local function load_code_invalid()
		return string.rep("\xFF", 5 * FUNCTION_ALIGNMENT)
	end

	local function load_code_platform()
		if jit.arch == "x64" then
			return load_code_x64()
		elseif jit.arch == "arm64" then
			return load_code_arm64()
		else
			return load_code_invalid()
		end
	end

	local function load_memory_linux(code)
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

	local function load_memory_mac_os(code)
		ffi.cdef([[
            void* mmap(void *addr, size_t length, int prot, int flags, int fd, size_t offset);
            int munmap(void *addr, size_t length);
            void sys_icache_invalidate(void *start, size_t length);
            void pthread_jit_write_protect_np(int enabled);
        ]])

		local PROT_READ = 0x01
		local PROT_WRITE = 0x02
		local PROT_EXEC = 0x04
		local MAP_PRIVATE = 0x02
		local MAP_ANONYMOUS = 0x1000
		local MAP_JIT = 0x0800

		local page = ffi_cast(
			u8_pointer_type,
			ffi.C.mmap(nil, #code, PROT_READ + PROT_WRITE + PROT_EXEC, MAP_PRIVATE + MAP_ANONYMOUS + MAP_JIT, -1, 0)
		)

		if page == ffi_cast(u8_pointer_type, -1) then
			error("failed to allocate code page for `f32`")
		end

		ffi.C.pthread_jit_write_protect_np(0)
		ffi.copy(page, code, #code)
		ffi.C.pthread_jit_write_protect_np(1)
		ffi.C.sys_icache_invalidate(page, #code)

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

	local function load_memory_platform(code)
		if jit.os == "Linux" then
			return load_memory_linux(code)
		elseif jit.os == "OSX" then
			return load_memory_mac_os(code)
		else
			return load_memory_invalid(code)
		end
	end

	local code = load_code_platform()
	local memory = load_memory_platform(code)

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
