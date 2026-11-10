open IR
open Definitions
open Emission
open Tree
open RustExtension

type reader = (x, Expression.t) Computation.reader

val computed_where_read : x node -> bool

module Conditions : Conditions.S with type expression = Expression.t

val test : Context.t -> reader -> x node -> int -> Expression.t option
val select : Expression.t -> (int list * Conditions.condition) list -> Conditions.condition
val compute : Context.t -> reader -> x node -> (Expression.t, Statement.t) Computation.t
