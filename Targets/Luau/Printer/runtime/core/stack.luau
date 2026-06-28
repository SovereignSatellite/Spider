-- SECTION stack_pool
local stack_pool = {
	[16] = {},
	[64] = {},
	[256] = {},
	[1024] = {},
	[4096] = {},
	[16384] = {},
}

-- SECTION stack_acquire
-- NEEDS stack_pool
local function stack_acquire(size_class: number)
	local stack = table.remove(stack_pool[size_class])

	return if stack == nil then table.create(size_class) else stack
end

-- SECTION stack_release
-- NEEDS stack_pool
local function stack_release(size_class: number, stack)
	table.insert(stack_pool[size_class], stack)
end
