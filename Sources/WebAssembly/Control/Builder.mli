open Types

val variables : signatures -> signature -> Wasm.Ast.func -> Variables.t
val build : signatures -> Variables.t -> Wasm.Ast.func -> Graph.t
