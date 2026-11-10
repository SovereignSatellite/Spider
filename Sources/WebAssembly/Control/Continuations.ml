type scratch = { arm : Bits.t; seen : Bits.t; stack : int Dynarray.t; frontier : int Dynarray.t }

let scratch () =
  { arm = Bits.create (); seen = Bits.create (); stack = Dynarray.create (); frontier = Dynarray.create () }

let grow graph scratch excluded entry =
  let { arm; seen; stack; frontier } = scratch in
  Bits.clear arm;
  Bits.clear seen;
  Dynarray.clear stack;
  Dynarray.clear frontier;
  ignore (Bits.add arm entry);
  ignore (Bits.add seen entry);
  let push_unseen id = if Bits.add seen id then Dynarray.add_last stack id in
  let joins id =
    (not (Bits.contains excluded id))
    && List.for_all
         (fun predecessor -> predecessor = id || Bits.contains arm predecessor)
         (Graph.block graph id).predecessors
  in
  let rec examine grew =
    if Dynarray.length stack > 0 then begin
      let id = Dynarray.pop_last stack in
      if joins id then begin
        ignore (Bits.add arm id);
        List.iter push_unseen (Graph.block graph id).successors;
        examine true
      end
      else begin
        Dynarray.add_last frontier id;
        examine grew
      end
    end
    else if grew then begin
      Dynarray.append stack frontier;
      Dynarray.clear frontier;
      examine false
    end
  in
  if Bits.contains excluded entry then ([ entry ], Bits.of_list [ List.hd (Graph.block graph entry).predecessors ])
  else begin
    List.iter push_unseen (Graph.block graph entry).successors;
    examine false;
    (Dynarray.to_list frontier, arm)
  end

let continuations graph scratch excluded head entry =
  if List.length (List.filter (fun predecessor -> predecessor <> entry) (Graph.block graph entry).predecessors) >= 2
  then [ (head, entry) ]
  else
    let points, arm = grow graph scratch excluded entry in
    List.concat_map
      (fun point ->
        List.filter_map
          (fun predecessor -> if Bits.contains arm predecessor then Some (predecessor, point) else None)
          (Graph.block graph point).predecessors)
      points

let arms graph scratch excluded head =
  List.map (continuations graph scratch excluded head) (Graph.block graph head).successors

let points_of arms = List.sort_uniq compare (List.concat_map (List.map snd) arms)
let sources arms = List.map (List.map fst) arms

let select graph arms =
  let points = points_of arms in
  let selection = Graph.selection graph Variable.a in
  let rerouted =
    List.map
      (List.map (fun (source, point) ->
           let assignment = Graph.assignment graph Variable.a (Graph.index_of point points) in
           Graph.replace_edge graph source point assignment;
           Graph.add_edge graph assignment selection;
           assignment))
      arms
  in
  List.iter (Graph.add_edge graph selection) points;
  (selection, rerouted)

(** When A assignments reach the points both from the arms and from elsewhere, some are an enclosing branch's. Excluding
    each one, or the C assignment that is its only predecessor, makes them this branch's points. *)
let select_point graph scratch head found points =
  let assignments =
    List.concat_map
      (fun point -> List.filter (Graph.assigns graph Variable.a) (Graph.block graph point).predecessors)
      points
  in
  let from_arms = List.concat (sources found) in
  let from_arm id = List.mem id from_arms in
  if List.exists from_arm assignments && not (List.for_all from_arm assignments) then
    let excluded =
      Bits.of_list
        (List.map
           (fun id ->
             match (Graph.block graph id).predecessors with
             | [ before ] when Graph.assigns graph Variable.c before -> before
             | _ -> id)
           assignments)
    in
    select graph (arms graph scratch excluded head)
  else select graph found

let continuation_point graph scratch head =
  let found = arms graph scratch (Bits.create ()) head in
  match points_of found with
  | [ point ] -> (point, sources found)
  | points -> select_point graph scratch head found points
