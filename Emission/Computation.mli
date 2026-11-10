open IR
open Allocator

type ('x, 'expression) reader = { value : 'x Link.t -> int -> 'expression; test : 'x Link.t -> int -> 'expression }

type ('expression, 'statement) t =
  | Statements of 'statement list
  | Each of (int -> 'expression option)
  | All of 'expression

val each_leaf : (int -> 'expression) -> ('expression, 'statement) t
val single : 'expression -> ('expression, 'statement) t
val only_leaf : ('expression, 'statement) t -> 'expression option
val output_leaves : 'x Layouts.t -> 'x Definitions.node -> ('x Definitions.link * int) list
val leaf_type_of : 'x Layouts.t -> 'x Link.t -> int -> Type.t
val label : ('x, string) RegionTable.t -> (string -> 'statement) -> 'x Definitions.region -> 'statement list
val leaf_values : 'x Layouts.t -> ('x, 'expression) reader -> 'x Link.t -> 'expression list
val input_values : 'x Layouts.t -> ('x, 'expression) reader -> 'x Node.t -> int -> 'expression list

(** Whether a node evaluates an input once while registers hold [held] of its output leaves: computed apart ([Each]),
    each leaf it holds evaluates an input of one leaf, and so does each leaf of an element it writes a leaf at a time.
*)
val reads_once : 'x Layouts.t -> 'x Node.t -> int -> int -> bool

val each_array : 'x Layouts.t -> Type.t -> (int -> 'statement) -> ('expression, 'statement) t
val element_value : 'x Layouts.t -> 'expression -> ('x, 'expression) reader -> 'x Link.t -> int -> 'expression

val calls :
  'x Layouts.t ->
  'expression ->
  (string -> 'expression) ->
  ('expression list -> 'expression) ->
  ('expression -> 'statement) ->
  ('x, 'expression) reader ->
  'x Node.t ->
  ('expression, 'statement) t

(** A sum's fields reach its slots through flows, so only its tag is computed. *)
val create_sum : 'x Layouts.t -> (int -> 'expression) -> Type.t -> int -> ('expression, 'statement) t

val runs : 'computed list -> (int list * 'computed) list
