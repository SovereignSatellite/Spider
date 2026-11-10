open IR
open Definitions
open Entities

val traps_on : 'x context -> Wasm.Ast.instr' -> ('x link list -> 'x link) option
val numeric : Wasm.Ast.instr' -> 'x link list -> 'x link
