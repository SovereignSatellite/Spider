open IR
open Allocator

module type Target = sig
  type x
  type frame
  type expression
  type statement

  val selects_once : bool
  val layouts : frame -> x Layouts.t
  val allocation : frame -> x Allocation.t
  val reader : frame -> (x, expression) Computation.reader
  val declare : frame -> Register.t -> statement list
  val compute : frame -> x Node.t -> (expression, statement) Computation.t
  val read : frame -> Register.t -> Type.t -> expression
  val path : Register.t -> expression
  val unused : expression
  val assign : expression list * expression list -> statement
  val discard : expression -> statement
  val if_ : expression * statement list * statement list -> statement
  val break_ : statement
  val select : expression -> (int list * statement list) list -> statement list
  val loop : statement list -> statement
end

module Make (Target : Target) : sig
  val reads_once : Target.x Layouts.t -> Target.x Node.t -> int -> int -> bool
  val body : Target.frame -> Target.x Region.t -> Target.statement list
end
