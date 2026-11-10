-- needs StackPool
local function stack_release(stack, size_class)
	table.insert(stack_pool[size_class], stack)
end
