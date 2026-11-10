-- needs LuauTryOutcome
const function luau_try(callee: (...any) -> ...any, ...: any): ...any
	return luau_try_outcome(pcall(callee, ...))
end
