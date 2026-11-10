open Types

val of_block : signatures -> Wasm.Ast.blocktype -> signature
val change : signatures -> Wasm.Ast.instr' -> signature
