open IR
open Definitions

val reads_fields : 'x Readers.table -> 'x node -> int -> bool
val values : 'x node -> int -> 'x link array option
val outside : 'x node -> int -> 'x link -> 'x link option
val root : 'x link -> 'x link
val variant : 'x type_arena -> Type.t -> int -> 'x link option
val variant_of : 'x type_arena -> 'x link -> int option
val knows : 'x link -> 'x link -> bool
val known : 'x type_arena -> 'x link -> int -> 'x link -> 'x link option
val tagged : 'x type_arena -> int -> 'x link -> 'x link array -> 'x link

val choose :
  'x type_arena ->
  'x link ->
  'x link array ->
  Type.t list ->
  ('x link array -> 'x link array) ->
  ('x link array -> 'x link array) ->
  'x link array

val choose_one :
  'x type_arena ->
  'x link ->
  'x link array ->
  Type.t ->
  ('x link array -> 'x link) ->
  ('x link array -> 'x link) ->
  'x link

val both : 'x type_arena -> 'x link -> 'x link -> 'x link
val either : 'x type_arena -> 'x link -> 'x link -> 'x link
val uniform : 'x node -> ('x link * 'x link array) option
val with_values : 'x type_arena -> 'x node -> int list -> 'x link array -> 'x link
val extend : 'x type_arena -> 'x node -> 'x link array -> 'x link

(** Each Match [matcher_of] gives, in order of first appearance, with the items it gives it for. *)
val grouped : ('item -> 'x node) -> 'item list -> ('x node * 'item list) list

(** A link in a region nested through uniform Matches, replaced by what [within] builds from the field carrying [outer]
    in; every Match on its path carries [outer], but a constant is made within instead. *)
type 'x route = { path : ('x node * int) list; link : 'x link; outer : 'x link; within : 'x link -> 'x link }

val route : 'x type_arena -> 'x Readers.table -> 'x route list -> unit
