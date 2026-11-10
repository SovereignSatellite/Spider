let boundary graph entry exit =
  let boundary = Bits.of_list (Graph.block graph entry).predecessors in
  List.iter (Bits.remove boundary) (Graph.block graph entry).successors;
  ignore (Bits.add boundary exit);
  boundary

let rec search next seen order id =
  if Bits.add seen id then id :: List.fold_left (search next seen) order (List.rev (next id)) else order

let is_loop graph found =
  match found with [ only ] -> List.mem only (Graph.block graph only).predecessors | _ -> found <> []

let components graph entry exit =
  let seen = boundary graph entry exit in
  let order = search (fun id -> (Graph.block graph id).successors) (Bits.copy seen) [] entry in
  let gather found id =
    let reached = search (fun id -> (Graph.block graph id).predecessors) seen [] id in
    if is_loop graph reached then reached :: found else found
  in
  List.rev (List.fold_left gather [] order)

(** Blocks restructuring adds are not in the region's set, so a block also counts as inside when a predecessor is. *)
let inside graph region id =
  Bits.contains region id || List.exists (Bits.contains region) (Graph.block graph id).predecessors

let select graph targets sources =
  let selection = Graph.selection graph Variable.c in
  List.iteri
    (fun index target ->
      List.iter
        (fun source ->
          let assignment = Graph.assignment graph Variable.c index in
          Graph.replace_edge graph source target assignment;
          Graph.add_edge graph assignment selection)
        (sources target);
      Graph.add_edge graph selection target)
    targets;
  selection

let entry_of graph entries =
  match entries with
  | [ entry ] -> entry
  | _ -> select graph entries (fun entry -> (Graph.block graph entry).predecessors)

let exit_of graph region exits =
  match exits with
  | [ exit ] -> exit
  | [] -> Graph.no_operation graph
  | _ -> select graph exits (fun exit -> List.filter (Bits.contains region) (Graph.block graph exit).predecessors)

let existing_latch graph region entry exit escapes =
  match (List.filter (inside graph region) (Graph.block graph entry).predecessors, escapes) with
  | [ repetition ], [ escape ] when repetition = escape && (Graph.block graph repetition).successors = [ exit; entry ]
    ->
      Some repetition
  | _ -> None

(** With one original entry, continuing also assigns C so that C is not live around the loop. The continuing blocks are
    found after the breaks are redirected, since until then an exit that also precedes the entry counts as inside. *)
let new_latch graph region single entry exit escapes =
  let latch = Graph.selection graph Variable.b in
  List.iter
    (fun source ->
      let assignment = Graph.assignment graph Variable.b 0 in
      Graph.replace_edge graph source exit assignment;
      Graph.add_edge graph assignment latch)
    escapes;
  List.iter
    (fun source ->
      let assignment = Graph.assignment graph Variable.b 1 in
      Graph.replace_edge graph source entry assignment;
      if single then begin
        let reset = Graph.assignment graph Variable.c 65535 in
        Graph.add_edge graph assignment reset;
        Graph.add_edge graph reset latch
      end
      else Graph.add_edge graph assignment latch)
    (List.filter (inside graph region) (Graph.block graph entry).predecessors);
  Graph.add_edge graph latch exit;
  Graph.add_edge graph latch entry;
  latch

let restructure graph blocks =
  let region = Bits.of_list blocks in
  let ids = List.sort compare blocks in
  let outside id = not (Bits.contains region id) in
  let entries = List.filter (fun id -> List.exists outside (Graph.block graph id).predecessors) ids in
  let exits =
    List.sort_uniq compare (List.concat_map (fun id -> List.filter outside (Graph.block graph id).successors) ids)
  in
  let entry = entry_of graph entries in
  let exit = exit_of graph region exits in
  let escapes = List.filter (fun id -> id <> exit && inside graph region id) (Graph.block graph exit).predecessors in
  match existing_latch graph region entry exit escapes with
  | Some latch -> (entry, latch)
  | None -> (entry, new_latch graph region (List.length entries = 1) entry exit escapes)

let between graph (entry, exit) = List.map (restructure graph) (components graph entry exit)

let rec levels graph loops =
  match loops with [] -> [] | _ -> loops @ levels graph (List.concat_map (between graph) loops)

let run graph exit = levels graph (between graph (0, exit))
