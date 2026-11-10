-- needs Thrown
const function throw(payload: any): ...any
	THROWN[1] = payload
	error(THROWN, 0)
end
