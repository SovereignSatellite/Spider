open IR
open Definitions
open Entities

val lift :
  ([> IR_Memory.t ] as 'a) context -> int -> Fields.key list -> Type.t -> Wasm.Ast.func -> 'a region -> 'a link array
