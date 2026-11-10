-- needs Bytes
-- needs FfiNew
local function lua_jit_bytes_new(length)
	local created, array = pcall(ffi_new, bytes, length)

	if not created then
		return nil
	end

	array.length = length

	return array
end
