open IR

val lift : ([> IR_Memory.t ] as 'a) TypeArena.t -> Wasm.Ast.module_ -> 'a CompilationUnit.t
val lift_labelled : ([> IR_Memory.t ] as 'a) TypeArena.t -> string -> 'a CompilationUnit.t * ('a, string) RegionTable.t
