-- SECTION environment
local environment = {}
local named = {}
local selected = nil

-- SECTION spectest
-- NEEDS environment
-- NEEDS from_bits_f32
-- NEEDS from_bits_f64
-- NEEDS into_bits_f32
-- NEEDS into_bits_f64
-- NEEDS memory_new
-- NEEDS table_new
do
	local spectest = {
		global_i32 = { 666 },
		global_i64 = { 666LL },
		global_f32 = { into_bits_f32(666.6) },
		global_f64 = { into_bits_f64(666.6) },

		table = rt_table_new({}, 10, 10),
		memory = rt_memory_new({}, 65536, 131072),
	}

	spectest.print = print

	function spectest.print_i32(argument)
		print(string.format("I32 `0x%08X`", argument))
	end

	function spectest.print_i64(argument)
		print(string.format("I64 `0x%016X`", argument))
	end

	function spectest.print_f32(argument)
		argument = from_bits_f32(argument)

		print(string.format("F32 `%g`", argument))
	end

	function spectest.print_f64(argument)
		argument = from_bits_f64(argument)

		print(string.format("F64 `%g`", argument))
	end

	function spectest.print_i32_f32(argument_1, argument_2)
		argument_2 = from_bits_f32(argument_2)

		print(string.format("I32 `0x%08X`, F32 `%g`", argument_1, argument_2))
	end

	function spectest.print_f64_f64(argument_1, argument_2)
		argument_1 = from_bits_f64(argument_1)
		argument_2 = from_bits_f64(argument_2)

		print(string.format("F64 `%g`, F64 `%g`", argument_1, argument_2))
	end

	environment.spectest = spectest
end

-- SECTION report_failure
local hn_failed_test_count = 0

function hn_report_failure(content, level, ...)
	local reason = string.format(content, ...)
	local report = debug.traceback(reason, level + 1)

	print(report)

	hn_failed_test_count = hn_failed_test_count + 1
end

-- SECTION assert_ok
-- NEEDS report_failure
function hn_assert_ok(callback)
	xpcall(callback, function(reason)
		hn_report_failure("%s", 2, reason)
	end)
end

-- SECTION assert_trap
-- NEEDS report_failure
function hn_assert_trap(reason, callback)
	if not pcall(callback) then
		return
	end

	hn_report_failure("should trap: %s", 2, reason)
end

-- SECTION assert_ref_null
-- NEEDS report_failure
function hn_assert_ref_null(source)
	if source == nil then
		return
	end

	hn_report_failure("`%s` should be null", 2, source)
end

-- SECTION assert_ref_extern
-- NEEDS report_failure
function hn_assert_ref_extern(source)
	if source ~= nil then
		return
	end

	hn_report_failure("source should be non null", 2)
end

-- SECTION assert_equal_i32
-- NEEDS report_failure
function hn_assert_equal_i32(target)
	return function(source)
		if type(source) ~= "number" then
			hn_report_failure("`%s` should be type `i32`", 2, source)
		elseif source ~= target then
			hn_report_failure("`%08X` (%s) should equal `%08X` (%s)", 2, source, source, target, target)
		end
	end
end

-- SECTION assert_equal_i64
-- NEEDS ffi
-- NEEDS report_failure
function hn_assert_equal_i64(target)
	return function(source)
		if not ffi.istype("int64_t", source) then
			hn_report_failure("`%s` should be type `i64`", 2, source)
		elseif source ~= target then
			hn_report_failure("`%016X` (%s) should equal `%016X` (%s)", 2, source, source, target, target)
		end
	end
end

-- SECTION is_f32_nan_canonical
-- NEEDS bit_and
function hn_is_f32_nan_canonical(source)
	return bit_and(source, 0x7FFFFFFF) == 0x7F800000
end

-- SECTION is_f32_nan_arithmetic
-- NEEDS bit_and
function hn_is_f32_nan_arithmetic(source)
	return bit_and(source, 0x7F800000) == 0x7F800000
end

-- SECTION assert_equal_f32
-- NEEDS from_bits_f32
-- NEEDS report_failure
function hn_assert_equal_f32(target)
	return function(source)
		if type(source) ~= "number" then
			hn_report_failure("`%s` should be type `f32`", 2, source)
		elseif source ~= target then
			hn_report_failure(
				"`%08X` (%s) should equal `%08X` (%s)",
				2,
				source,
				from_bits_f32(source),
				target,
				from_bits_f32(target)
			)
		end
	end
end

-- SECTION is_f64_nan_canonical
-- NEEDS bit_and
function hn_is_f64_nan_canonical(source)
	return bit_and(source, 0x7FFFFFFFFFFFFFFFLL) == 0x7FF0000000000000LL
end

-- SECTION is_f64_nan_arithmetic
-- NEEDS bit_and
function hn_is_f64_nan_arithmetic(source)
	return bit_and(source, 0x7FF0000000000000LL) == 0x7FF0000000000000LL
end

-- SECTION assert_equal_f64
-- NEEDS ffi
-- NEEDS from_bits_f64
-- NEEDS report_failure
function hn_assert_equal_f64(target)
	return function(source)
		if not ffi.istype("int64_t", source) then
			hn_report_failure("`%s` should be type `f64`", 2, source)
		elseif source ~= target then
			hn_report_failure(
				"`%016X` (%s) should equal `%016X` (%s)",
				2,
				source,
				from_bits_f64(source),
				target,
				from_bits_f64(target)
			)
		end
	end
end
