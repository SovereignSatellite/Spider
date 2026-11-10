-- needs Thrown
local function throw(payload)
	THROWN[1] = payload
	error(THROWN, 0)
end
