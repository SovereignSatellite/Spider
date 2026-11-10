open Emission

(* Luau: Luau has types, floor division, if-expressions and constants. *)
module LuauType = struct
  type t =
    | Named of Name.t
    | Generic of Name.t * t list
    | Function of t list * t list
    | Union of t list
    | Table of (Name.t * t) list
    | Sequence of t

  let union members =
    let flat = List.concat_map (fun added -> match added with Union inner -> inner | _ -> [ added ]) members in
    match List.sort_uniq compare flat with [] -> Named (Text "nil") | [ single ] -> single | sorted -> Union sorted

  let table fields = Table (List.stable_sort (fun (left, _) (right, _) -> compare left right) fields)
end

module LuauOperator = struct
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

module Recursive = struct
  module rec Expression : sig
    type t =
      | Nil
      | Integer of int64
      | Number of int64
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
  end =
    Expression

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
  end =
    Statement
end

module Expression = struct
  include Recursive.Expression

  let grouped items = match items with [] -> Nil | [ item ] -> item | _ -> Table (items, [])
end

module Statement = Recursive.Statement
