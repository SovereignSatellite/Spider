-- needs FfiTypeof
local bytes = ffi_typeof("struct { int32_t length; uint8_t data[?]; }")
