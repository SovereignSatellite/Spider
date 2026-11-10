open IR

type x = [ IR_Memory.t | IR_LuaJIT.t ]

val extension : x Definitions.extension
