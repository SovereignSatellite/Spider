if hn_failed_test_count == 0 then
	return
end

error("FAILED " .. hn_failed_test_count .. " TESTS", 0)
