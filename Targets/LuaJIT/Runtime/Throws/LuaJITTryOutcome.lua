-- needs Thrown
local function lua_jit_try_outcome(returned, ...)
	if returned then
		return 0, nil, ...
	elseif ... ~= THROWN then
		error(..., 0)
	end

	return 1, THROWN[1]
end
