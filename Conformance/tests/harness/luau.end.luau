if #hn_failed_test_list == 0 then
	return
end

local report = table.concat(hn_failed_test_list, "\n")

print(report)
error("SOME TESTS FAILED", 0)
