open Emission

module LuaJITOperator : sig
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

module rec Expression : sig
  type t =
    | Nil
    | Integer of int64
    | Number of int64  (** Finite, and held as bits so that equality tells 0 and -0 apart. *)
    (* LuaJIT: the FFI has long literals, and LuaJIT is untyped. *)
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

  val grouped : t list -> t
end

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
end
