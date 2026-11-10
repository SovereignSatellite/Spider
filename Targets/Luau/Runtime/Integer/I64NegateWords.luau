const function i64_negate_words(low: number, high: number): (number, number)
	if low == 0 then
		return 0, (0x1_0000_0000 - high) % 0x1_0000_0000
	end

	return 0x1_0000_0000 - low, 0xFFFF_FFFF - high
end
