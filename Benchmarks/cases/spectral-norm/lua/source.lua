-- Adapted from spectralnorm-lua-1 under ../LICENSE.txt.

local ITERATION_COUNT = 10

local function inverseMatrixValue(row, column)
	local diagonal = row + column - 1
	return 1 / (diagonal * (diagonal - 1) * 0.5 + row)
end

local function multiplyByMatrix(source, destination, vectorLength)
	for row = 1, vectorLength do
		local total = 0
		for column = 1, vectorLength do
			total = total + source[column] * inverseMatrixValue(row, column)
		end
		destination[row] = total
	end
end

local function multiplyByTranspose(source, destination, vectorLength)
	for row = 1, vectorLength do
		local total = 0
		for column = 1, vectorLength do
			total = total + source[column] * inverseMatrixValue(column, row)
		end
		destination[row] = total
	end
end

local function multiplyAtAv(source, destination, temporary, vectorLength)
	multiplyByMatrix(source, temporary, vectorLength)
	multiplyByTranspose(temporary, destination, vectorLength)
end

local function benchmark(vectorLength)
	local vectorU = {}
	local vectorV = {}
	local temporary = {}
	for index = 1, vectorLength do
		vectorU[index] = 1
	end

	for _ = 1, ITERATION_COUNT do
		multiplyAtAv(vectorU, vectorV, temporary, vectorLength)
		multiplyAtAv(vectorV, vectorU, temporary, vectorLength)
	end

	local vectorProduct = 0
	local squaredVector = 0
	for index = 1, vectorLength do
		vectorProduct = vectorProduct + vectorU[index] * vectorV[index]
		squaredVector = squaredVector + vectorV[index] * vectorV[index]
	end

	return math.sqrt(vectorProduct / squaredVector)
end
