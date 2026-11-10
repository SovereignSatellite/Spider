open IR

type x = [ IR_Memory.t | IR_Rust.t ]

val extension : x Definitions.extension
