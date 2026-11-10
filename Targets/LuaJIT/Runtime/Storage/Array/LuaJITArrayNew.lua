-- needs TableNew
local function lua_jit_array_new(length)
	local created, array = pcall(table_new, length, 1)

	if not created then
		return nil
	end

	array.length = length

	return array
end
