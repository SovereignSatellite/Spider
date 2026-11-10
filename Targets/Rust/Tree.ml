open Emission

module RustType = struct
  type t =
    | Named of Name.t
    | Generic of Name.t * t list
    | Function of t list * t
    | Pointer of t
    | Slice of t
    | Tuple of t list
end

module RustOperator = struct
  type binary =
    | Add
    | Subtract
    | Multiply
    | Divide
    | Remainder
    | BitwiseAnd
    | BitwiseOr
    | BitwiseExclusiveOr
    | Equal
    | NotEqual
    | LessThan
    | LessThanEqual
    | And
    | Or

  type unary = Negate | Not | Dereference | Reference
end

module Recursive = struct
  module rec Expression : sig
    type t =
      | Integer of int64 * string
      | Bytes of string
      | Name of Name.t
      | Generic of Name.t * RustType.t list
      | Call of t * t list
      | Method of t * string * t list
      | Field of t * Name.t
      | Binary of RustOperator.binary * t * t
      | Unary of RustOperator.unary * t
      | Tuple of t list
      | Array of t list
      | Struct of Name.t * (Name.t * t) list
      | BooleanToInteger of t
      | IfElse of t * t * t
      | Match of t * (int list * t) list
      | Cast of t * RustType.t
      | Transmute of t * RustType.t
      | Closure of t
  end =
    Expression

  and Statement : sig
    type t =
      | Local of Name.t * RustType.t * Expression.t
      | Assign of Expression.t list * Expression.t list
      | Call of Expression.t
      | If of Expression.t * t list * t list
      | Loop of t list
      | Break
      | Return of Expression.t list
      | Match of Expression.t * (int list * t list) list
      | Comment of string
  end =
    Statement
end

module Expression = struct
  include Recursive.Expression

  let grouped items = match items with [ item ] -> item | _ -> Tuple items
end

module Statement = Recursive.Statement

module Item = struct
  type t =
    | Function of Name.t * (Name.t * RustType.t) list * RustType.t * Statement.t list
    | Type of Name.t * (Name.t * RustType.t) list
    | Source of string
end
