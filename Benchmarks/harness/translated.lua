module()

local benchmarkClosure = assert(
	rt_export_map["benchmark"],
	"translated module did not export benchmark"
)

local function benchmark(strength)
	return benchmarkClosure[1](benchmarkClosure, strength)
end
