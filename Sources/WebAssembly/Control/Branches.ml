let through_dummy graph point sources =
  let dummy = Graph.no_operation graph in
  List.iter (fun source -> Graph.replace_edge graph source point dummy) sources;
  Graph.add_edge graph dummy point

let merge graph point arm = match arm with _ :: _ :: _ -> through_dummy graph point arm | _ -> ()

let fill_skipped_arms graph head point =
  List.iter
    (fun successor -> if successor = point then through_dummy graph point [ head ])
    (Graph.block graph head).successors

let restructure graph scratch head =
  let point, arms = Continuations.continuation_point graph scratch head in
  List.iter (merge graph point) arms;
  fill_skipped_arms graph head point;
  point

let rec next_head graph entry exit =
  if entry = exit then None
  else
    match List.filter (fun successor -> successor <> entry) (Graph.block graph entry).successors with
    | [ next ] -> next_head graph next exit
    | _ -> Some entry

let push graph stack entry exit =
  match next_head graph entry exit with Some head -> (head, exit) :: stack | None -> stack

let rec regions graph scratch stack =
  match stack with [] -> () | (head, exit) :: rest -> restructure_region graph scratch head exit rest

and restructure_region graph scratch head exit rest =
  let point = restructure graph scratch head in
  regions graph scratch
    (List.fold_left
       (fun stack entry -> push graph stack entry point)
       (push graph rest point exit) (Graph.block graph head).successors)

let run graph exit = regions graph (Continuations.scratch ()) (push graph [] 0 exit)
