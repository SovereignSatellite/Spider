open Tree

let rec reads name (expression : Expression.t) =
  match expression with
  | Nil | Integer _ | Number _ | Bytes _ | Function _ -> false
  | Name read -> read = name
  (* Luau: Luau has types, constants, casts and if-expressions. *)
  | Field (operand, _) | Unary (_, operand) | Cast (operand, _) -> reads name operand
  | Index (first, second) | Binary (_, first, second) -> read_by name [ first; second ]
  | Call (callee, arguments) -> read_by name (callee :: arguments)
  (* Luau: Luau has types, constants, casts and if-expressions. *)
  | IfElse (condition, value, otherwise) -> read_by name [ condition; value; otherwise ]
  | Table (items, fields) -> read_by name (items @ List.map snd fields)
  | Record fields -> read_by name (List.map snd fields)

and read_by name expressions = List.exists (reads name) expressions

let rec mentions name (statement : Statement.t) =
  match statement with
  (* Luau: Luau has types, constants, casts and if-expressions. *)
  | Local (_, _, value) -> Option.fold ~none:false ~some:(reads name) value
  | Assign (targets, values) -> read_by name (targets @ values)
  | Call value -> reads name value
  | If (condition, body, otherwise) -> reads name condition || mentioned_in name body || mentioned_in name otherwise
  | Do body | Loop body -> mentioned_in name body
  | Return values -> read_by name values
  (* Luau: Luau has types, constants, casts and if-expressions. *)
  | Const _ | Break | Type _ | Source _ | Comment _ -> false

and mentioned_in name block = List.exists (mentions name) block

(* Luau: Luau has types, constants, casts and if-expressions. *)

(** The register's leaf of an assignment as a declaration of its own: the values of several leaves read only registers,
    so they may be taken in any order, and none reads this one. *)
let taken_apart name leaf_type targets values =
  let leaves = List.combine targets values in
  let declaration = Statement.Local (name, leaf_type, Some (List.assoc (Expression.Name name) leaves)) in
  match List.split (List.remove_assoc (Expression.Name name) leaves) with
  | [], [] -> [ declaration ]
  | targets, values -> [ declaration; Assign (targets, values) ]

(* Luau: Luau has types, constants, casts and if-expressions. *)

(** The block declaring the register at its first mention, if that is an assignment not reading it and every other
    mention follows within the block, as then nothing sees an earlier value, across a loop's back edge either. *)
let rec declared name leaf_type (block : Statement.t list) =
  match block with
  | Assign (targets, values) :: rest
    when List.mem (Expression.Name name) targets && List.compare_lengths targets values = 0 && not (read_by name values)
    ->
      (* Luau: Luau has types, constants, casts and if-expressions. *)
      Some (taken_apart name leaf_type targets values @ rest)
  | statement :: rest when not (mentions name statement) ->
      Option.map (List.cons statement) (declared name leaf_type rest)
  | statement :: rest when not (mentioned_in name rest) ->
      Option.map (fun inner -> inner :: rest) (declared_within name leaf_type statement)
  | _ -> None

(* Luau: Luau has types, constants, casts and if-expressions. *)
and declared_within name leaf_type statement =
  match statement with
  | If (condition, body, otherwise) when not (reads name condition || mentioned_in name otherwise) ->
      (* Luau: Luau has types, constants, casts and if-expressions. *)
      Option.map (fun body -> Statement.If (condition, body, otherwise)) (declared name leaf_type body)
  | If (condition, body, otherwise) when not (reads name condition || mentioned_in name body) ->
      Option.map (fun otherwise -> Statement.If (condition, body, otherwise)) (declared name leaf_type otherwise)
  | Loop body -> Option.map (fun body -> Statement.Loop body) (declared name leaf_type body)
  | If _ | Do _ | Local _ | Const _ | Assign _ | Call _ | Break | Return _ | Type _ | Source _ | Comment _ -> None

let rec placed (statements : Statement.t list) =
  match statements with
  | [] -> []
  (* Luau: Luau has types, constants, casts and if-expressions. *)
  | (Local (name, leaf_type, None) as local) :: rest ->
      let rest = placed rest in
      Option.value (declared name leaf_type rest) ~default:(local :: rest)
  | statement :: rest -> statement :: placed rest
