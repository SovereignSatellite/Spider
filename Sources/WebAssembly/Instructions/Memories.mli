open IR
open Definitions
open Entities

val memory : Wasm.Ast.idx -> Fields.key
val size : 'x context -> 'x State.t -> Wasm.Ast.idx -> 'x link list * 'x State.t
val grow : 'x context -> 'x State.t -> Wasm.Ast.idx -> 'x link -> 'x link list * 'x State.t
val drop : 'x context -> 'x State.t -> int -> 'x link list * 'x State.t

val load :
  ([> IR_Memory.t ] as 'a) context ->
  'a State.t ->
  Wasm.Ast.idx ->
  Wasm.Ast.loadop ->
  'a link ->
  'a link list * 'a State.t

val store :
  ([> IR_Memory.t ] as 'a) context ->
  'a State.t ->
  Wasm.Ast.idx ->
  Wasm.Ast.storeop ->
  'a link ->
  'a link ->
  'a link list * 'a State.t
