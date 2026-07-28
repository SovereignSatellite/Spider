-- Adapted from fannkuchredux-lua-1 under ../LICENSE.txt.

local function benchmark(permutationLength)
	local permutation = {}
	local workingPermutation = {}
	local rotationCounts = {}
	local checksum = 0
	local maximumFlipCount = 0
	local sign = 1

	for value = 1, permutationLength do
		permutation[value] = value
		workingPermutation[value] = value
		rotationCounts[value] = value
	end

	while true do
		local firstValue = permutation[1]
		if firstValue ~= 1 then
			for index = 2, permutationLength do
				workingPermutation[index] = permutation[index]
			end

			local flipCount = 1
			while true do
				local nextFirstValue = workingPermutation[firstValue]
				if nextFirstValue == 1 then
					checksum = checksum + sign * flipCount
					maximumFlipCount = math.max(maximumFlipCount, flipCount)
					break
				end

				workingPermutation[firstValue] = firstValue
				if firstValue >= 4 then
					local lowerIndex = 2
					local upperIndex = firstValue - 1
					repeat
						workingPermutation[lowerIndex], workingPermutation[upperIndex] =
							workingPermutation[upperIndex], workingPermutation[lowerIndex]
						lowerIndex = lowerIndex + 1
						upperIndex = upperIndex - 1
					until lowerIndex >= upperIndex
				end

				firstValue = nextFirstValue
				flipCount = flipCount + 1
			end
		end

		if sign == 1 then
			permutation[2], permutation[1] = permutation[1], permutation[2]
			sign = -1
		else
			permutation[2], permutation[3] = permutation[3], permutation[2]
			sign = 1

			for rotationIndex = 3, permutationLength do
				local rotationsRemaining = rotationCounts[rotationIndex]
				if rotationsRemaining ~= 1 then
					rotationCounts[rotationIndex] = rotationsRemaining - 1
					break
				end
				if rotationIndex == permutationLength then
					return checksum * 128 + maximumFlipCount
				end

				rotationCounts[rotationIndex] = rotationIndex
				local rotatedValue = permutation[1]
				for destinationIndex = 1, rotationIndex do
					permutation[destinationIndex] = permutation[destinationIndex + 1]
				end
				permutation[rotationIndex + 1] = rotatedValue
			end
		end
	end
end
