open IR
open Allocator

module type Spelling = sig
  type expression

  val truth : bool -> expression
  val equal : expression -> int -> expression
  val both : expression -> expression -> expression
  val either : expression -> expression -> expression
  val if_else : expression -> expression -> expression -> expression
  val not_ : expression -> expression
end

module type S = sig
  type expression
  type condition = Known of bool | Holds of expression | Is of expression * int

  val holds : expression -> int -> expression
  val render : condition -> expression
  val choose : (int -> condition) -> condition -> condition -> condition

  val reader :
    'x Layouts.t ->
    'x Allocation.t ->
    (expression -> (int list * condition) list -> condition) ->
    (('x, expression) Computation.reader -> 'x Node.t -> int -> expression option) ->
    (('x, expression) Computation.reader -> 'x Node.t -> (expression, 'statement) Computation.t) ->
    (Register.t -> Type.t -> expression) ->
    (Type.t -> expression) ->
    ('x, expression) Computation.reader

  val choose_value :
    'x Layouts.t ->
    (expression -> (int list * expression) list -> expression) ->
    ('x, expression) Computation.reader ->
    'x Node.t ->
    expression
end

val known : 'x Allocation.t -> 'x Link.t -> int -> bool option

module Make (Spelling : Spelling) : S with type expression = Spelling.expression
