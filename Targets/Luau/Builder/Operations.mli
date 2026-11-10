open IR
open Definitions
open Emission
open Tree
open LuauExtension

type reader = (x, Expression.t) Computation.reader

val call_helper : Context.t -> string -> Expression.t list -> Expression.t
val computed_where_read : x node -> bool

module Conditions : Conditions.S with type expression = Expression.t

val halved : Expression.t -> ((int -> Expression.t) -> 'built -> 'built -> 'built) -> (int list * 'built) list -> 'built
val test : Context.t -> reader -> x node -> int -> Expression.t option
val select : Expression.t -> (int list * Conditions.condition) list -> Conditions.condition
val compute : Context.t -> reader -> x node -> (Expression.t, Statement.t) Computation.t
