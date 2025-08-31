-- SECTION environment
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
-- NEEDS memory_new
-- NEEDS table_new
-- NEEDS vector_create
local environment = {}
local named = {}
local selected = nil

do
	local spectest = {
		global_i32 = { 666 },
		global_i64 = { rt_create_i64_from_u32(666, 0) },
		global_f32 = { vector_create(666.6, 0, 0) },
		global_f64 = { 666.6 },

		table = rt_table_new(10, 10),
		memory = rt_memory_new(1, 2),
	}

	spectest.print = print

	function spectest.print_i32(argument: number)
		print(string.format("I32 `0x%08X`", argument))
	end

	function spectest.print_i64(argument: number)
		local argument_1, argument_2 = rt_create_u32_from_i64(argument)

		print(string.format("I64 `0x%08X%08X`", argument_2, argument_1))
	end

	function spectest.print_f32(argument: vector)
		print(string.format("F32 `%g`", argument.x))
	end

	function spectest.print_f64(argument: number)
		print(string.format("F64 `%g`", argument))
	end

	function spectest.print_i32_f32(argument_1: number, argument_2: vector)
		print(string.format("I32 `0x%08X`, F32 `%g`", argument_1, argument_2.x))
	end

	function spectest.print_f64_f64(argument_1: number, argument_2: number)
		print(string.format("F64 `%g`, F64 `%g`", argument_1, argument_2))
	end

	environment.spectest = spectest
end

-- SECTION assert_trap
local function hn_assert_trap(message: string, callback: () -> ())
	if not pcall(callback) then
		return
	end

	error("should trap: " .. message, 2)
end

-- SECTION assert_ref_null
local function hn_assert_ref_null(source: unknown)
	if source ~= nil then
		error(string.format("`%s` should be null", source), 2)
	end
end

-- SECTION assert_ref_extern
local function hn_assert_ref_extern(source: unknown)
	if source == nil then
		error("source should be non null", 2)
	end
end

-- SECTION assert_equal_i32
local function hn_assert_equal_i32(target: number): (unknown) -> ()
	return function(source: unknown)
		if type(source) ~= "number" then
			error(string.format("`%*` should be type `i32`", source), 2)
		end

		if source ~= target then
			error(string.format("`%08X` (%*) should equal `%08X` (%*)", source, source, target, target), 2)
		end
	end
end

-- SECTION assert_equal_i64
-- NEEDS convert_u64_to_f64
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
local function hn_assert_equal_i64(target_1: number, target_2: number): (unknown) -> ()
	local target = rt_create_i64_from_u32(target_1, target_2)

	return function(source: unknown)
		if type(source) ~= "number" then
			error(string.format("`%*` should be type `i64`", source), 2)
		end

		local source_1, source_2 = rt_create_u32_from_i64(source)

		if source_1 ~= target_1 or source_2 ~= target_2 then
			local source = rt_convert_u64_to_f64(source)
			local target = rt_convert_u64_to_f64(target)

			error(
				string.format(
					"`%08X%08X` (%*) should equal `%08X%08X` (%*)",
					source_2,
					source_1,
					source,
					target_2,
					target_1,
					target
				),
				2
			)
		end
	end
end

-- SECTION is_f32_nan_canonical
-- NEEDS is_f64_nan_canonical
function hn_is_f32_nan_canonical(source: vector): boolean
	return hn_is_f64_nan_canonical(source.x)
end

-- SECTION is_f32_nan_arithmetic
-- NEEDS is_f64_nan_arithmetic
function hn_is_f32_nan_arithmetic(source: vector): boolean
	return hn_is_f64_nan_arithmetic(source.x)
end

-- SECTION assert_equal_f32
-- NEEDS transmute_f32_to_i32
-- NEEDS transmute_i32_to_f32
local function hn_assert_equal_f32(target: number): (unknown) -> ()
	local target = rt_transmute_i32_to_f32(target)

	return function(source: unknown)
		if type(source) ~= "vector" then
			error(string.format("`%*` should type `f32`", source), 2)
		end

		local source_1 = rt_transmute_f32_to_i32(source)
		local target_1 = rt_transmute_f32_to_i32(target)

		if source_1 ~= target_1 then
			local source = source.x
			local target = target.x

			error(string.format("`%08X` (%*) should equal `%08X` (%*)", source_1, source, target_1, target), 2)
		end
	end
end

-- SECTION is_f64_nan_canonical
-- NEEDS create_u32_from_i64
function hn_is_f64_nan_canonical(source: number): boolean
	local source_1, source_2 = rt_create_u32_from_i64(source)

	return source_1 == 0 and (source_2 == 0x7FF80000 or source_2 == 0xFFF80000)
end

-- SECTION is_f64_nan_arithmetic
-- NEEDS is_f64_nan_canonical
function hn_is_f64_nan_arithmetic(source: number): boolean
	return source ~= source and not hn_is_f64_nan_canonical(source)
end

-- SECTION assert_equal_f64
-- NEEDS create_f64_from_u32
-- NEEDS create_u32_from_i64
-- NEEDS transmute_f64_to_i64
local function hn_assert_equal_f64(target_1: number, target_2: number): (unknown) -> ()
	local target = rt_create_f64_from_u32(target_1, target_2)

	return function(source: unknown)
		if type(source) ~= "number" then
			error(string.format("`%*` should type `f64`", source), 2)
		end

		local source_1, source_2 = rt_create_u32_from_i64(rt_transmute_f64_to_i64(source))

		if source_1 ~= target_1 or source_2 ~= target_2 then
			error(
				string.format(
					"`%08X%08X` (%*) should equal `%08X%08X` (%*)",
					source_2,
					source_1,
					source,
					target_2,
					target_1,
					target
				),
				2
			)
		end
	end
end
