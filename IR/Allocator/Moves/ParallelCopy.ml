module Registers = Register.Map

type state = {
  predecessors : Register.t Registers.t;
  locations : Register.t Registers.t;
  leaf_types : IR.Type.t Registers.t;
}

let rec cycle state start current members =
  match Registers.find current state.predecessors with
  | next when next = start -> List.rev (current :: members)
  | next -> cycle state start next (current :: members)

let rec run state ready remaining moves =
  match (ready, remaining) with
  | destination :: rest, _ -> copy_into state destination rest remaining moves
  | [], destination :: rest when Registers.find_opt destination state.locations = Some destination ->
      swap_cycle state destination rest moves
  | [], _ :: rest -> run state [] rest moves
  | [], [] -> List.rev moves

and copy_into state destination ready remaining moves =
  let source = Registers.find destination state.predecessors in
  let location = Registers.find source state.locations in
  let freed = if location = source && Registers.mem source state.predecessors then source :: ready else ready in
  run
    { state with locations = Registers.add source destination state.locations }
    freed remaining
    (Move.Copy (location, Registers.find source state.leaf_types, destination) :: moves)

and swap_cycle state destination remaining moves =
  let members = cycle state destination destination [] in
  let first = Registers.find destination state.leaf_types in
  let rec pairwise = function first :: (second :: _ as rest) -> (first, second) :: pairwise rest | _ -> [] in
  let swaps =
    List.map
      (fun (current, next) -> Move.Swap (current, first, next, Registers.find next state.leaf_types))
      (pairwise members)
  in
  let locations =
    List.fold_left
      (fun located member -> Registers.add (Registers.find member state.predecessors) member located)
      state.locations members
  in
  run { state with locations } [] remaining (List.rev_append swaps moves)

let sequentialize copies =
  let predecessors = Registers.of_list (List.map (fun (source, destination, _) -> (destination, source)) copies) in
  let locations = Registers.of_list (List.map (fun (source, _, _) -> (source, source)) copies) in
  let leaf_types = Registers.of_list (List.map (fun (source, _, leaf_type) -> (source, leaf_type)) copies) in
  let destinations = List.map (fun (_, destination, _) -> destination) copies in
  run { predecessors; locations; leaf_types }
    (List.filter (fun destination -> not (Registers.mem destination locations)) destinations)
    destinations []
