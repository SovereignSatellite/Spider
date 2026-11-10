-- needs StackPool
const function stack_acquire(size_class: number): { any }
	return table.remove(stack_pool[size_class]) or table.create(size_class)
end
