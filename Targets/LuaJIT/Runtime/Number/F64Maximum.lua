local function f64_maximum(left, right)
	if left > right then
		return left
	elseif right > left then
		return right
	elseif left == right then
		return 1 / left > 0 and left or right
	end

	return left + right
end
