open Emission

module LuaJITOperator = struct
  type binary =
    | Add
    | Subtract
    | Multiply
    | Divide
    | Modulo
    | Equal
    | NotEqual
    | LessThan
    | LessThanEqual
    | GreaterThanEqual
    | And
    | Or

  type unary = Negate | Not
end

module Recursive = struct
  module rec Expression : sig
    type t =
      | Nil
      | Integer of int64
      (* LuaJIT: the FFI has long literals, and LuaJIT is untyped. *)
      | Number of int64
      | Long of int64
      | Bytes of string
      | Name of Name.t
      | Call of t * t list
      | Field of t * Name.t
      | Index of t * t
      | Binary of LuaJITOperator.binary * t * t
      | Unary of LuaJITOperator.unary * t
      | Table of t list * (Name.t * t) list
      | Record of (Name.t * t) list
      | Function of Name.t list * Statement.t list
  end =
    Expression

  and Statement : sig
    type t =
      | Local of Name.t * Expression.t option
      | Assign of Expression.t list * Expression.t list
      | Call of Expression.t
      | If of Expression.t * t list * t list
      | Do of t list
      | Loop of t list
      | Break
      | Return of Expression.t list
      | Source of string
      | Comment of string
  end =
    Statement
end

module Expression = struct
  include Recursive.Expression

  let grouped items = match items with [] -> Nil | [ item ] -> item | _ -> Table (items, [])
end

module Statement = Recursive.Statement
