open Tree

let rec reads name (expression : Expression.t) =
  match expression with
  (* LuaJIT: the FFI has long literals. *)
  | Nil | Integer _ | Number _ | Long _ | Bytes _ | Function _ -> false
  | Name read -> read = name
  | Field (operand, _) | Unary (_, operand) -> reads name operand
  | Index (first, second) | Binary (_, first, second) -> read_by name [ first; second ]
  | Call (callee, arguments) -> read_by name (callee :: arguments)
  | Table (items, fields) -> read_by name (items @ List.map snd fields)
  | Record fields -> read_by name (List.map snd fields)

and read_by name expressions = List.exists (reads name) expressions

let rec mentions name (statement : Statement.t) =
  match statement with
  | Local (_, value) -> Option.fold ~none:false ~some:(reads name) value
  | Assign (targets, values) -> read_by name (targets @ values)
  | Call value -> reads name value
  | If (condition, body, otherwise) -> reads name condition || mentioned_in name body || mentioned_in name otherwise
  | Do body | Loop body -> mentioned_in name body
  | Return values -> read_by name values
  | Break | Source _ | Comment _ -> false

and mentioned_in name block = List.exists (mentions name) block

(** The register's leaf of an assignment as a declaration of its own: the values of several leaves read only registers,
    so they may be taken in any order, and none reads this one. *)
let taken_apart name targets values =
  let leaves = List.combine targets values in
  let declaration = Statement.Local (name, Some (List.assoc (Expression.Name name) leaves)) in
  match List.split (List.remove_assoc (Expression.Name name) leaves) with
  | [], [] -> [ declaration ]
  | targets, values -> [ declaration; Assign (targets, values) ]

(** The block declaring the register at its first mention, if that is an assignment not reading it and every other
    mention follows within the block, as then nothing sees an earlier value, across a loop's back edge either. *)
let rec declared name (block : Statement.t list) =
  match block with
  | Assign (targets, values) :: rest
    when List.mem (Expression.Name name) targets && List.compare_lengths targets values = 0 && not (read_by name values)
    ->
      Some (taken_apart name targets values @ rest)
  | statement :: rest when not (mentions name statement) -> Option.map (List.cons statement) (declared name rest)
  | statement :: rest when not (mentioned_in name rest) ->
      Option.map (fun inner -> inner :: rest) (declared_within name statement)
  | _ -> None

and declared_within name statement =
  match statement with
  | If (condition, body, otherwise) when not (reads name condition || mentioned_in name otherwise) ->
      Option.map (fun body -> Statement.If (condition, body, otherwise)) (declared name body)
  | If (condition, body, otherwise) when not (reads name condition || mentioned_in name body) ->
      Option.map (fun otherwise -> Statement.If (condition, body, otherwise)) (declared name otherwise)
  | Loop body -> Option.map (fun body -> Statement.Loop body) (declared name body)
  | If _ | Do _ | Local _ | Assign _ | Call _ | Break | Return _ | Source _ | Comment _ -> None

let rec placed (statements : Statement.t list) =
  match statements with
  | [] -> []
  | (Local (name, None) as local) :: rest ->
      let rest = placed rest in
      Option.value (declared name rest) ~default:(local :: rest)
  | statement :: rest -> statement :: placed rest
