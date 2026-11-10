open Emission

(* Luau: Luau has types, floor division, if-expressions and constants. *)
module LuauType : sig
  type t =
    | Named of Name.t
    | Generic of Name.t * t list
    | Function of t list * t list
    | Union of t list
    | Table of (Name.t * t) list
    | Sequence of t

  val union : t list -> t
  val table : (Name.t * t) list -> t
end

module LuauOperator : sig
  type binary =
    | Add
    | Subtract
    | Multiply
    | Divide
    (* Luau: Luau has types, floor division, if-expressions and constants. *)
    | FloorDivide
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
    | Bytes of string
    | Name of Name.t
    | Call of t * t list
    | Field of t * Name.t
    | Index of t * t
    | Binary of LuauOperator.binary * t * t
    | Unary of LuauOperator.unary * t
    (* Luau: Luau has types, floor division, if-expressions and constants. *)
    | IfElse of t * t * t
    | Cast of t * LuauType.t
    | Table of t list * (Name.t * t) list
    | Record of (Name.t * t) list
    | Function of bool * (Name.t * LuauType.t) list * LuauType.t list * Statement.t list

  val grouped : t list -> t
end

and Statement : sig
  type t =
    (* Luau: Luau has types, floor division, if-expressions and constants. *)
    | Local of Name.t * LuauType.t * Expression.t option
    | Const of Name.t * LuauType.t option * Expression.t
    | Assign of Expression.t list * Expression.t list
    | Call of Expression.t
    | If of Expression.t * t list * t list
    | Do of t list
    | Loop of t list
    | Break
    | Return of Expression.t list
    (* Luau: Luau has types, floor division, if-expressions and constants. *)
    | Type of Name.t * LuauType.t
    | Source of string
    | Comment of string
end
