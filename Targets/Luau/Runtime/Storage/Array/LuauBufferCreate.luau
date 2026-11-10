-- needs BufferCreate
const function luau_buffer_create(length: number): buffer?
	const created, array = pcall(buffer_create, length)

	if not created then
		return nil
	end

	return array
end
