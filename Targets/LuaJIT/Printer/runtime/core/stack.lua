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
require("table.new")

local function stack_acquire(size_class)
	local stack = table.remove(stack_pool[size_class])

	if stack == nil then
		return table.new(size_class, 0)
	else
		return stack
	end
end

-- SECTION stack_release
-- NEEDS stack_pool
local function stack_release(size_class, stack)
	table.insert(stack_pool[size_class], stack)
end
