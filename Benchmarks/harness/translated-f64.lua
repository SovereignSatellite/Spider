module()

local benchmarkClosure = assert(
	rt_export_map["benchmark"],
	"translated module did not export benchmark"
)

local function benchmark(strength)
	return from_bits_f64(benchmarkClosure[1](benchmarkClosure, strength))
end
