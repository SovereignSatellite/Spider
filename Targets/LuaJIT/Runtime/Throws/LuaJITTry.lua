-- needs LuaJITTryOutcome
local function lua_jit_try(callee, ...)
	return lua_jit_try_outcome(pcall(callee, ...))
end
