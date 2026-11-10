-- needs Thrown
const function luau_try_outcome(returned: boolean, ...: any): ...any
	if returned then
		return 0, nil, ...
	elseif ... ~= THROWN then
		error(..., 0)
	end

	return 1, THROWN[1]
end
