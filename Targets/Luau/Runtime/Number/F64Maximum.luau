const function f64_maximum(left: number, right: number): number
	if left > right then
		return left
	elseif right > left then
		return right
	elseif left == right then
		return if 1 / left > 0 then left else right
	end

	return left + right
end
