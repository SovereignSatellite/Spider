-- needs Array
const function luau_array_new<T>(length: number, value: T): Array<T>?
	const created, array: any = pcall(table.create, length, value)

	if not created then
		return nil
	end

	array.length = length

	return array
end
