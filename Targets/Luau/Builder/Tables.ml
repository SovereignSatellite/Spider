(* Luau: bodies sit in constant tables, which the Luau compiler resolves statically. *)
open IR

type table = Assigned | Constant of int

(** Each table is a local of the module and an upvalue of each body naming it, and Luau allows 200 of each. *)
let constant_tables = 64

let table_size = 128

(** Kosaraju's strongly connected components of the bodies the root reaches, each after every component it names. *)
let components root =
  let callees = RegionTable.create 64 in
  let callers = RegionTable.create 64 in
  let rec finish finished body =
    if RegionTable.mem callees body then finished
    else begin
      RegionTable.replace callees body (Regions.named body);
      body :: List.fold_left finish finished (RegionTable.find callees body)
    end
  in
  let finished = finish [] root in
  let callers_of body = Option.value (RegionTable.find_opt callers body) ~default:[] in
  List.iter
    (fun body ->
      List.iter
        (fun callee -> RegionTable.replace callers callee (body :: callers_of callee))
        (RegionTable.find callees body))
    finished;
  let taken = RegionSet.create 64 in
  let rec collect members body =
    if RegionSet.add taken body then List.fold_left collect (body :: members) (callers_of body) else members
  in
  let components = List.filter (fun members -> not (List.is_empty members)) (List.map (collect []) finished) in
  (List.rev components, callees)

let add_level levels callees body =
  let level =
    List.fold_left
      (fun level callee ->
        Option.fold ~none:level ~some:(fun named -> max level (named + 1)) (RegionTable.find_opt levels callee))
      0 (RegionTable.find callees body)
  in
  RegionTable.replace levels body level

let group_by key items =
  List.map
    (fun group -> (group, List.filter (fun item -> key item = group) items))
    (List.sort_uniq compare (List.map key items))

let rec chunks size items =
  match items with [] -> [] | _ -> List.take size items :: chunks size (List.drop size items)

let group root =
  let components, callees = components root in
  let levels = RegionTable.create 64 in
  let tables = RegionTable.create 64 in
  List.iter
    (fun bodies ->
      match bodies with
      | [ body ] when not (List.memq body (RegionTable.find callees body)) -> add_level levels callees body
      | _ -> List.iter (fun body -> RegionTable.replace tables body Assigned) bodies)
    components;
  group_by (RegionTable.find levels) (List.filter (RegionTable.mem levels) (List.concat components))
  |> List.concat_map (fun (_, bodies) -> chunks table_size bodies)
  |> List.iteri (fun index bodies ->
      let table = if index < constant_tables then Constant index else Assigned in
      List.iter (fun body -> RegionTable.replace tables body table) bodies);
  tables
