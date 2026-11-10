open IR
open Definitions

val producers : 'x Readers.table -> 'x region -> 'x link -> ('x node * 'x region) list option
val building : int -> ('x node * 'x region) list -> ('x node * 'x region) list
val extensible : ('x node * 'x region) list -> bool

val rebuild :
  'x type_arena ->
  'x Readers.table ->
  Type.t list ->
  ('x node -> 'x link array -> 'x link array) ->
  'x link array ->
  'x region ->
  'x link ->
  'x link array

(** Runs within a Repeat's redefinition, which has already given the region the step type read here. [fields] must give
    every variant a [Sum.FromI32] builds the same fields, as that node's variants share its values. *)
val retype :
  'x type_arena ->
  'x Readers.table ->
  (int -> 'x link array -> 'x link array) ->
  'x link array ->
  'x region ->
  'x link ->
  'x link
