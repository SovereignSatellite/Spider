open Emission

module RustType : sig
  type t =
    | Named of Name.t
    | Generic of Name.t * t list
    | Function of t list * t
    | Pointer of t
    | Slice of t
    | Tuple of t list
end

module RustOperator : sig
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

  val grouped : t list -> t
end

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
end

module Item : sig
  type t =
    | Function of Name.t * (Name.t * RustType.t) list * RustType.t * Statement.t list
    | Type of Name.t * (Name.t * RustType.t) list
    | Source of string
end
