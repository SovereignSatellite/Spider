-- SECTION environment
local environment = {}
local named = {}
local selected = nil

-- SECTION spectest
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
-- NEEDS environment
-- NEEDS memory_new
-- NEEDS narrow_f64
-- NEEDS table_new
-- NEEDS widen_f32
do
	local spectest = {
		global_i32 = { 666 },
		global_i64 = { rt_create_i64_from_u32(666, 0) },
		global_f32 = { rt_widen_f32(666.6) },
		global_f64 = { 666.6 },

		table = rt_table_new({}, 10, 10),
		memory = rt_memory_new({}, 65536, 131072),
	}

	spectest.print = print

	function spectest.print_i32(argument)
		print(string.format("I32 `0x%08X`", argument))
	end

	function spectest.print_i64(argument)
		local argument_1, argument_2 = rt_create_u32_from_i64(argument)

		print(string.format("I64 `0x%08X%08X`", argument_2, argument_1))
	end

	function spectest.print_f32(argument)
		local argument = rt_widen_f32(argument)

		print(string.format("F32 `%g`", argument))
	end

	function spectest.print_f64(argument)
		print(string.format("F64 `%g`", argument))
	end

	function spectest.print_i32_f32(argument_1, argument_2)
		local argument_2 = rt_widen_f32(argument_2)

		print(string.format("I32 `0x%08X`, F32 `%g`", argument_1, argument_2))
	end

	function spectest.print_f64_f64(argument_1, argument_2)
		print(string.format("F64 `%g`, F64 `%g`", argument_1, argument_2))
	end

	environment.spectest = spectest
end

-- SECTION report_failure
local hn_failed_test_list = {}

function hn_report_failure(content: string, level: number, ...: any)
	local reason = string.format(content, ...)
	local report = debug.traceback(reason, level + 1)

	table.insert(hn_failed_test_list, report)
end

-- SECTION assert_ok
-- NEEDS report_failure
function hn_assert_ok(callback: () -> ())
	xpcall(callback, function(reason: unknown)
		hn_report_failure("%*", 2, reason)
	end)
end

-- SECTION assert_trap
-- NEEDS report_failure
function hn_assert_trap(reason: string, callback: () -> ())
	if not pcall(callback) then
		return
	end

	hn_report_failure("should trap: %*", 2, reason)
end

-- SECTION assert_ref_null
-- NEEDS report_failure
function hn_assert_ref_null(source: unknown)
	if source == nil then
		return
	end

	hn_report_failure("`%*` should be null", 2, source)
end

-- SECTION assert_ref_extern
-- NEEDS report_failure
function hn_assert_ref_extern(source: unknown)
	if source ~= nil then
		return
	end

	hn_report_failure("source should be non null", 2)
end

-- SECTION assert_equal_i32
-- NEEDS report_failure
function hn_assert_equal_i32(target: number): (unknown) -> ()
	return function(source: unknown)
		if type(source) ~= "number" then
			hn_report_failure("`%*` should be type `i32`", 2, source)
		elseif source ~= target then
			hn_report_failure("`%08X` (%*) should equal `%08X` (%*)", 2, source, source, target, target)
		end
	end
end

-- SECTION assert_equal_i64
-- NEEDS convert_u64_to_f64
-- NEEDS create_i64_from_u32
-- NEEDS create_u32_from_i64
-- NEEDS report_failure
function hn_assert_equal_i64(target_1: number, target_2: number): (unknown) -> ()
	local target = rt_create_i64_from_u32(target_1, target_2)

	return function(source)
		if type(source) ~= "number" then
			hn_report_failure("`%*` should be type `i64`", 2, source)

			return
		end

		local source_1, source_2 = rt_create_u32_from_i64(source)

		if source_1 ~= target_1 or source_2 ~= target_2 then
			local source = rt_convert_u64_to_f64(source)
			local target = rt_convert_u64_to_f64(target)

			hn_report_failure(
				"`%08X%08X` (%*) should equal `%08X%08X` (%*)",
				2,
				source_2,
				source_1,
				source,
				target_2,
				target_1,
				target
			)
		end
	end
end

-- SECTION is_f32_nan_canonical
-- NEEDS is_f64_nan_canonical
-- NEEDS widen_f32
function hn_is_f32_nan_canonical(source)
	local source = rt_widen_f32(source)

	return hn_is_f64_nan_canonical(source)
end

-- SECTION is_f32_nan_arithmetic
-- NEEDS is_f64_nan_arithmetic
-- NEEDS widen_f32
function hn_is_f32_nan_arithmetic(source)
	local source = rt_widen_f32(source)

	return hn_is_f64_nan_arithmetic(source)
end

-- SECTION assert_equal_f32
-- NEEDS report_failure
-- NEEDS widen_f32
function hn_assert_equal_f32(target: number): (unknown) -> ()
	return function(source)
		if type(source) ~= "number" then
			hn_report_failure("`%*` should be type `f32`", 2, source)

			return
		end

		if source ~= target then
			local source_1 = rt_widen_f32(source)
			local target_1 = rt_widen_f32(target)

			hn_report_failure("`%08X` (%*) should equal `%08X` (%*)", 2, source, source_1, target, target_1)
		end
	end
end

-- SECTION is_f64_nan_canonical
-- NEEDS create_u32_from_i64
function hn_is_f64_nan_canonical(source)
	local source_1, source_2 = rt_create_u32_from_i64(source)

	return source_1 == 0 and (source_2 == 0x7FF80000 or source_2 == 0xFFF80000)
end

-- SECTION is_f64_nan_arithmetic
-- NEEDS is_f64_nan_canonical
function hn_is_f64_nan_arithmetic(source)
	return source ~= source and not hn_is_f64_nan_canonical(source)
end

-- SECTION assert_equal_f64
-- NEEDS create_f64_from_u32
-- NEEDS create_u32_from_i64
-- NEEDS report_failure
-- NEEDS transmute_f64_to_i64
function hn_assert_equal_f64(target_1: number, target_2: number): (unknown) -> ()
	local target = rt_create_f64_from_u32(target_1, target_2)

	return function(source)
		if type(source) ~= "number" then
			hn_report_failure("`%*` should be type `f64`", 2, source)

			return
		end

		local source_1, source_2 = rt_create_u32_from_i64(rt_transmute_f64_to_i64(source))

		if source_1 ~= target_1 or source_2 ~= target_2 then
			hn_report_failure(
				"`%08X%08X` (%*) should equal `%08X%08X` (%*)",
				2,
				source_2,
				source_1,
				source,
				target_2,
				target_1,
				target
			)
		end
	end
end
