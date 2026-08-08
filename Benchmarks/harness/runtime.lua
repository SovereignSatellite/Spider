local strengthArgument, expectedResultArgument = ...

local strength = assert(tonumber(strengthArgument), "expected an integer strength")
local expectedResult = assert(tonumber(expectedResultArgument), "expected a result")
local tolerance = math.max(0.000000000001, math.abs(expectedResult) * 0.000000000001)

local function assertExpectedResult(result)
	assert(math.abs(result - expectedResult) <= tolerance, "unexpected benchmark result")
end

assertExpectedResult(benchmark(strength))
for _ = 1, 3 do
	local startTime = os.clock()
	local result = benchmark(strength)
	local elapsedSeconds = os.clock() - startTime
	assertExpectedResult(result)
	print(string.format("%.9f", elapsedSeconds))
end
