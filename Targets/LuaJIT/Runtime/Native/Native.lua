-- needs Ffi
local native = (function()
	local code

	if jit.arch == "arm64" then
		code = {
			"\x00\x00\x27\x1E\x21\x00\x27\x1E\x00\x28\x21\x1E\x00\x00\x26\x1E\xC0\x03\x5F\xD6",
			"\x00\x00\x27\x1E\x21\x00\x27\x1E\x00\x38\x21\x1E\x00\x00\x26\x1E\xC0\x03\x5F\xD6",
			"\x00\x00\x27\x1E\x21\x00\x27\x1E\x00\x08\x21\x1E\x00\x00\x26\x1E\xC0\x03\x5F\xD6",
			"\x00\x00\x27\x1E\x21\x00\x27\x1E\x00\x18\x21\x1E\x00\x00\x26\x1E\xC0\x03\x5F\xD6",
			"\x00\x00\x27\x1E\x00\xC0\x21\x1E\x00\x00\x26\x1E\xC0\x03\x5F\xD6",
			"\x00\x10\xC0\x5A\xC0\x03\x5F\xD6",
			"\x00\x00\xC0\x5A\x00\x10\xC0\x5A\xC0\x03\x5F\xD6",
			"\x00\x10\xC0\xDA\xC0\x03\x5F\xD6",
			"\x00\x00\xC0\xDA\x00\x10\xC0\xDA\xC0\x03\x5F\xD6",
			"\x00\x00\x22\x9E\x00\x00\x26\x1E\xC0\x03\x5F\xD6",
			"\x00\x00\x23\x9E\x00\x00\x26\x1E\xC0\x03\x5F\xD6",
		}
	elseif jit.arch == "x64" and jit.os == "Windows" then
		code = {
			"\x66\x0F\x6E\xC1\x66\x0F\x6E\xCA\xF3\x0F\x58\xC1\x66\x0F\x7E\xC0\xC3",
			"\x66\x0F\x6E\xC1\x66\x0F\x6E\xCA\xF3\x0F\x5C\xC1\x66\x0F\x7E\xC0\xC3",
			"\x66\x0F\x6E\xC1\x66\x0F\x6E\xCA\xF3\x0F\x59\xC1\x66\x0F\x7E\xC0\xC3",
			"\x66\x0F\x6E\xC1\x66\x0F\x6E\xCA\xF3\x0F\x5E\xC1\x66\x0F\x7E\xC0\xC3",
			"\x66\x0F\x6E\xC1\xF3\x0F\x51\xC0\x66\x0F\x7E\xC0\xC3",
			"\x0F\xBD\xC1\x74\x04\x83\xF0\x1F\xC3\xB8\x20\x00\x00\x00\xC3",
			"\x0F\xBC\xC1\x74\x01\xC3\xB8\x20\x00\x00\x00\xC3",
			"\x48\x0F\xBD\xC1\x74\x04\x83\xF0\x3F\xC3\xB8\x40\x00\x00\x00\xC3",
			"\x48\x0F\xBC\xC1\x74\x01\xC3\xB8\x40\x00\x00\x00\xC3",
			"\x0F\x57\xC0\xF3\x48\x0F\x2A\xC1\x66\x0F\x7E\xC0\xC3",
			"\x0F\x57\xC0\x48\x85\xC9\x78\x0A\xF3\x48\x0F\x2A\xC1\x66\x0F\x7E\xC0\xC3\x48\x89\xC8\x48\xD1\xE8\x83\xE1\x01\x48\x09\xC8\xF3\x48\x0F\x2A\xC0\xF3\x0F\x58\xC0\x66\x0F\x7E\xC0\xC3",
		}
	elseif jit.arch == "x64" then
		code = {
			"\x66\x0F\x6E\xC7\x66\x0F\x6E\xCE\xF3\x0F\x58\xC1\x66\x0F\x7E\xC0\xC3",
			"\x66\x0F\x6E\xC7\x66\x0F\x6E\xCE\xF3\x0F\x5C\xC1\x66\x0F\x7E\xC0\xC3",
			"\x66\x0F\x6E\xC7\x66\x0F\x6E\xCE\xF3\x0F\x59\xC1\x66\x0F\x7E\xC0\xC3",
			"\x66\x0F\x6E\xC7\x66\x0F\x6E\xCE\xF3\x0F\x5E\xC1\x66\x0F\x7E\xC0\xC3",
			"\x66\x0F\x6E\xC7\xF3\x0F\x51\xC0\x66\x0F\x7E\xC0\xC3",
			"\x0F\xBD\xC7\x74\x04\x83\xF0\x1F\xC3\xB8\x20\x00\x00\x00\xC3",
			"\x0F\xBC\xC7\x74\x01\xC3\xB8\x20\x00\x00\x00\xC3",
			"\x48\x0F\xBD\xC7\x74\x04\x83\xF0\x3F\xC3\xB8\x40\x00\x00\x00\xC3",
			"\x48\x0F\xBC\xC7\x74\x01\xC3\xB8\x40\x00\x00\x00\xC3",
			"\x0F\x57\xC0\xF3\x48\x0F\x2A\xC7\x66\x0F\x7E\xC0\xC3",
			"\x0F\x57\xC0\x48\x85\xFF\x78\x0A\xF3\x48\x0F\x2A\xC7\x66\x0F\x7E\xC0\xC3\x48\x89\xF8\x48\xD1\xE8\x83\xE7\x01\x48\x09\xF8\xF3\x48\x0F\x2A\xC0\xF3\x0F\x58\xC0\x66\x0F\x7E\xC0\xC3",
		}
	else
		error("LuaJIT's native code runs on x64 and arm64, not " .. jit.arch)
	end

	local size = 64 * #code
	local page

	if jit.os == "Windows" then
		ffi.cdef([[
			void *VirtualAlloc(void *address, size_t size, uint32_t type, uint32_t protection);
			int VirtualProtect(void *address, size_t size, uint32_t protection, uint32_t *old);
			void *GetCurrentProcess(void);
			int FlushInstructionCache(void *process, const void *address, size_t size);
		]])

		local MEM_COMMIT_RESERVE, PAGE_READWRITE, PAGE_EXECUTE_READ = 0x3000, 0x04, 0x20

		page = ffi.cast("uint8_t *", ffi.C.VirtualAlloc(nil, size, MEM_COMMIT_RESERVE, PAGE_READWRITE))
		assert(page ~= nil, "VirtualAlloc failed")

		for index, function_code in ipairs(code) do
			ffi.copy(page + 64 * (index - 1), function_code, #function_code)
		end

		assert(ffi.C.VirtualProtect(page, size, PAGE_EXECUTE_READ, ffi.new("uint32_t[1]")) ~= 0, "VirtualProtect failed")
		ffi.C.FlushInstructionCache(ffi.C.GetCurrentProcess(), page, size)
	elseif jit.os == "Linux" or jit.os == "OSX" then
		ffi.cdef([[
			void *mmap(void *address, size_t size, int protection, int flags, int file, int64_t offset);
			int mprotect(void *address, size_t size, int protection);
			void pthread_jit_write_protect_np(int enabled);
			void sys_icache_invalidate(void *address, size_t size);
		]])

		local PROT_READ, PROT_WRITE, PROT_EXEC, MAP_PRIVATE = 0x01, 0x02, 0x04, 0x02
		local MAP_ANONYMOUS = jit.os == "OSX" and 0x1000 or 0x20
		local apple_silicon = jit.os == "OSX" and jit.arch == "arm64"
		local MAP_JIT = apple_silicon and 0x0800 or 0
		local protection = apple_silicon and PROT_READ + PROT_WRITE + PROT_EXEC or PROT_READ + PROT_WRITE

		page = ffi.cast("uint8_t *", ffi.C.mmap(nil, size, protection, MAP_PRIVATE + MAP_ANONYMOUS + MAP_JIT, -1, 0))
		assert(page ~= ffi.cast("uint8_t *", -1), "mmap failed")

		if apple_silicon then
			ffi.C.pthread_jit_write_protect_np(0)
		end

		for index, function_code in ipairs(code) do
			ffi.copy(page + 64 * (index - 1), function_code, #function_code)
		end

		if apple_silicon then
			ffi.C.pthread_jit_write_protect_np(1)
			ffi.C.sys_icache_invalidate(page, size)
		else
			assert(ffi.C.mprotect(page, size, PROT_READ + PROT_EXEC) == 0, "mprotect failed")
		end
	else
		error("LuaJIT's native code runs on Linux, macOS and Windows, not " .. jit.os)
	end

	local binary = ffi.typeof("int32_t (*)(int32_t, int32_t)")
	local unary = ffi.typeof("int32_t (*)(int32_t)")
	local long = ffi.typeof("int64_t (*)(int64_t)")
	local converted = ffi.typeof("int32_t (*)(int64_t)")

	local function at(index, signature)
		return ffi.cast(signature, page + 64 * index)
	end

	return {
		f32_add = at(0, binary),
		f32_subtract = at(1, binary),
		f32_multiply = at(2, binary),
		f32_divide = at(3, binary),
		f32_square_root = at(4, unary),
		i32_leading_zeros = at(5, unary),
		i32_trailing_zeros = at(6, unary),
		i64_leading_zeros = at(7, long),
		i64_trailing_zeros = at(8, long),
		i64_convert_s_to_f32 = at(9, converted),
		i64_convert_u_to_f32 = at(10, converted),
	}
end)()
