-- needs StackPool
const function stack_release(stack: { any }, size_class: number)
	table.insert(stack_pool[size_class], stack)
end
