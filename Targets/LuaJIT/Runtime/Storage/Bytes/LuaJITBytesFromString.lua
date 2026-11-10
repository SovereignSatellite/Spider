-- needs Bytes
-- needs FfiCopy
-- needs FfiNew
local function lua_jit_bytes_from_string(text)
	local array = ffi_new(bytes, #text)

	array.length = #text
	ffi_copy(array.data, text, #text)

	return array
end
