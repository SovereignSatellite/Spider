open IR
open Definitions
open Entities

(** Array lengths are unsigned [I32], so 65536 pages of 65536 bytes (2^32) do not fit. *)
val most_pages : int

val instantiate :
  ([> IR_Memory.t ] as 'a) context ->
  Wasm.Ast.module_ ->
  Fields.key list list ->
  Type.t list ->
  'a link ->
  'a link array
