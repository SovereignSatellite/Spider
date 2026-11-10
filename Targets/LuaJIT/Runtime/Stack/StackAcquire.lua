-- needs StackPool
-- needs TableNew
local function stack_acquire(size_class)
	return table.remove(stack_pool[size_class]) or table_new(size_class, 0)
end
